pub mod fitbit;
pub mod google;

use actix::prelude::{Addr};
use actix_web::{AsyncResponder, FutureResponse, HttpRequest, http::header, HttpResponse, Path, Query, FromRequest};
use actix_web::middleware::identity::{RequestIdentity};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use futures::{Future, future::{ok, result}};
use uuid::Uuid;

use super::AppState;
use super::db::{CreateToken, DbExecutor, UpsertUser};

#[derive(Serialize)]
pub struct OAuthToken {
    service: String,
    access_token: String,
    expiration: DateTime<Utc>,
    refresh_token: String,
    scopes: Vec<String>,
    user_id: String,
    email: Option<String>
}

#[derive(Debug)]
pub enum OAuthError {
    DotEnv(dotenv::Error),
    Reqwest(reqwest::Error),
    TokenError(String),
    Error(String)
}

impl From<dotenv::Error> for OAuthError {
    fn from(e: dotenv::Error) -> Self {
        OAuthError::DotEnv(e)
    }
}

impl From<reqwest::Error> for OAuthError {
    fn from(e: reqwest::Error) -> Self {
        OAuthError::Reqwest(e)
    }
}

fn urlencode(to_encode: &str) -> String {
  utf8_percent_encode(to_encode, DEFAULT_ENCODE_SET).to_string()
}

pub fn start_oauth(service: String) -> Result<String, OAuthError> {
    match service.as_str() {
        "fitbit" => fitbit::redirect(),
        "google" => google::redirect(),
        _ => Err(OAuthError::Error(String::from("Bad service")))
    }
}

pub fn start_oauth_route(req: &HttpRequest<AppState>) -> HttpResponse {
    let service = Path::<String>::extract(&req).unwrap_or(Path::<String>::from("not-a-service".to_owned()));
    let redirect_url = start_oauth(service.into_inner());
    match redirect_url {
        Ok(url) => HttpResponse::Found().header(header::LOCATION, url).finish(),
        Err(_) => HttpResponse::BadRequest().body("Bad request")
    }
}

fn try_login(db: &Addr<DbExecutor>, maybe_userid: &Option<String>, token: &OAuthToken) -> FutureResponse<String> {
    if let Some(id) = maybe_userid {
        info!("Already logged in as: {}", id);
        return Box::new(result::<String, actix_web::Error>(Ok(id.to_string())));
    } else if token.service == "google" {
        db.send(UpsertUser {
            email: match &token.email {
                Some(email) => email,
                None => unimplemented!()
            }.clone(),
            g_sub: token.user_id.clone()
        })
        .from_err()
        .and_then(|res| match res {
            Ok(user) => {
                info!("Logged in as {} ({})", user.id, user.email);
                Ok(user.id.to_string())
            },
            Err(_e) => unimplemented!()
        })
        .responder()
    } else {
        unimplemented!()
    }
}

pub fn oauth_callback(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let req = req.clone();
    let db = req.state().db.clone();

    let service = Path::<String>::extract(&req).unwrap_or(Path::<String>::from("not-a-service".to_owned()));
    let query = Query::<HashMap<String, String>>::extract(&req).expect("No query string!");

    let token = match query.get("code") {
        Some(c) => match service.into_inner().as_str() {
            "fitbit" => fitbit::oauth_flow(c),
            "google" => google::oauth_flow(c),
            _ => Err(OAuthError::Error(String::from("Bad service")))
        },
        None => Err(OAuthError::TokenError(String::from("No token!")))
    };


    match token {
        Ok(t) => {
            // If not logged in, and we have a new sub, log in or create user
            try_login(&db, &req.identity(), &t)
            .and_then(move |user_id| {
                req.remember(user_id.clone());
                info!("{}, {:?}", user_id, req.identity());
                db.send(CreateToken {
                    access_token: t.access_token,
                    access_token_expiry: t.expiration,
                    refresh_token: t.refresh_token,
                    service: t.service,
                    service_userid: t.user_id,
                    user_id: Uuid::parse_str(&user_id).unwrap()
                })
                .from_err()
            })
            .and_then(|res| match res {
                Ok(token) => Ok(HttpResponse::Ok().json(token)),
                Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string()))
            })
            .responder()
        },
        Err(e) => ok(HttpResponse::InternalServerError().body(format!("{:?}", e))).responder()
    }
}