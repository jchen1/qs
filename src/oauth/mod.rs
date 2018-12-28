mod fitbit;
mod google;

use actix_web::{AsyncResponder, http::header, FutureResponse, HttpResponse, Path, Query, State};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use futures::{Future, future::result};
use uuid::Uuid;

use super::AppState;
use super::db::{CreateToken};

#[derive(Serialize)]
pub struct OAuthToken {
    service: String,
    access_token: String,
    expiration: DateTime<Utc>,
    refresh_token: String,
    scopes: Vec<String>,
    user_id: String
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

pub fn oauth_start(service: Path<String>) -> HttpResponse {
    let redirect_url = match service.into_inner().as_str() {
        "fitbit" => fitbit::redirect(),
        "google" => google::redirect(),
        _ => Err(OAuthError::Error(String::from("Bad service")))
    };

    match redirect_url {
        Ok(url) => HttpResponse::Found().header(header::LOCATION, url).finish(),
        Err(_) => HttpResponse::BadRequest().body("Bad request")
    }
}

pub fn oauth_callback(state: State<AppState>, service: Path<(String)>, query: Query<HashMap<String, String>>) -> FutureResponse<HttpResponse> {
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
            state.db.send(CreateToken {
                access_token: t.access_token,
                access_token_expiry: t.expiration,
                refresh_token: t.refresh_token,
                service: t.service,
                service_userid: t.user_id,
                // LOL
                user_id: Uuid::parse_str("21415c9a-47b1-4a8e-acef-565f9e5dc043").unwrap()
            })
            .from_err()
            .and_then(|res| match res {
                Ok(token) => Ok(HttpResponse::Ok().json(token)),
                Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string()))
            })
            .responder()
        },
        // todo use the error
        Err(e) => Box::new(result::<HttpResponse, actix_web::Error>(Ok(HttpResponse::BadRequest().body(format!("{:?}", e)))))
    }
}