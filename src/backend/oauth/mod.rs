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
use std::fmt::{self, Display};

use super::AppState;
use crate::db::{self, UpsertToken, DbExecutor, UpsertUser};

#[derive(Serialize)]
pub struct OAuthToken {
    pub service: String,
    pub access_token: String,
    pub expiration: DateTime<Utc>,
    pub refresh_token: String,
    scopes: Vec<String>,
    pub user_id: String,
    email: Option<String>
}

impl From<db::Token> for OAuthToken {
    fn from(t: db::Token) -> Self {
        OAuthToken {
            service: t.service,
            access_token: t.access_token,
            expiration: t.access_token_expiry,
            refresh_token: t.refresh_token,
            // todo
            scopes: vec![],
            user_id: t.service_userid,
            email: None
        }
    }
}

#[derive(Debug)]
pub enum OAuthError {
    DotEnv(dotenv::Error),
    Reqwest(reqwest::Error),
    TokenError(String),
    Error(String)
}

impl Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OAuthError::DotEnv(e) => write!(f, "{}", e),
            OAuthError::Reqwest(e) => write!(f, "{}", e),
            OAuthError::TokenError(e) => write!(f, "{}", e),
            OAuthError::Error(e) => write!(f, "{}", e)
        }
    }
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

pub fn refresh_token(token: OAuthToken) -> Result<OAuthToken, OAuthError> {
    match token.service.as_str() {
        "fitbit" => fitbit::refresh(token),
        "google" => google::refresh(token),
        _ => Err(OAuthError::Error(String::from("Bad service")))
    }
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
    } else 
    if token.service == "google" {
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
                db.send(UpsertToken {
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
                Ok(_) => Ok(HttpResponse::Found().header(header::LOCATION, "/").finish()),
                Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string()))
            })
            .responder()
        },
        Err(e) => ok(HttpResponse::InternalServerError().body(format!("{:?}", e))).responder()
    }
}

pub fn logout(req: &HttpRequest<AppState>) -> HttpResponse {
    req.forget();
    HttpResponse::Found().header(header::LOCATION, "/").finish()
}