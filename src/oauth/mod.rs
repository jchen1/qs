mod fitbit;

use actix_web::{http::header, HttpResponse, Path, Query};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct OAuthToken {
    service: String,
    access_token: String,
    expiration: DateTime<Utc>,
    refresh_token: String,
    scopes: Vec<String>,
    user_id: String
}

pub enum OAuthError {
    DotEnv(dotenv::Error),
    Reqwest(reqwest::Error),
    TokenError(String)
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
    match service.into_inner().as_str() {
        "fitbit" => {
            HttpResponse::Found()
                .header(header::LOCATION, fitbit::redirect())
                .finish()
        }
        _ => HttpResponse::BadRequest().body("Bad request")
    }
}

pub fn oauth_callback(service: Path<(String)>, query: Query<HashMap<String, String>>) -> HttpResponse {
    let token = match query.get("code") {
        Some(c) => match service.into_inner().as_str() {
            "fitbit" => fitbit::oauth_flow(c),
            _ => Err(OAuthError::TokenError(String::from("Bad service")))
        },
        None => Err(OAuthError::TokenError(String::from("No token!")))
    };

    match token {
        Ok(t) => HttpResponse::Ok().json(t),
        // todo use the error
        Err(_e) => HttpResponse::BadRequest().body("Bad request")
    }
}