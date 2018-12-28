use actix_web::{http::header, HttpResponse, Path, Query};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use time::Duration;

static FITBIT_CLIENT_ID: &'static str = "22DFW9";
static FITBIT_REDIRECT_URI: &'static str = "http://localhost:8080/oauth/fitbit/callback";
static FITBIT_EXPIRATION_MS: i32 = 604800;

#[derive(Deserialize)]
struct FitbitCallbackResponse {
    access_token: String,
    expires_in: u32,
    refresh_token: String,
    user_id: String,
    scope: String
}

#[derive(Serialize)]
struct OAuthToken {
    service: String,
    access_token: String,
    expiration: DateTime<Utc>,
    refresh_token: String,
    scopes: Vec<String>,
    user_id: String
}

impl From<FitbitCallbackResponse> for OAuthToken {
    fn from(fcr: FitbitCallbackResponse) -> Self {
        OAuthToken {
            service: "fitbit".to_string(),
            access_token: fcr.access_token,
            refresh_token: fcr.refresh_token,
            user_id: fcr.user_id,
            scopes: fcr.scope.split(" ").map(String::from).collect(),
            expiration: Utc::now() + Duration::seconds(fcr.expires_in as i64)
        }
    }
}

enum OAuthError {
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

pub fn oauth_start(service: Path<(String)>) -> HttpResponse {
    let scopes = ["activity", "heartrate", "location", "profile", "sleep", "weight"].join(" ");
    let redirect_uri = format!("https://www.fitbit.com/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&expires_in={}",
        urlencode(FITBIT_CLIENT_ID),
        urlencode(FITBIT_REDIRECT_URI),
        urlencode(&scopes),
        // 1 hour
        FITBIT_EXPIRATION_MS);

    HttpResponse::Found()
        .header(header::LOCATION, redirect_uri)
        .finish()
}

fn oauth_flow(code: &str) -> Result<OAuthToken, OAuthError> {
    dotenv::dotenv().ok();

    let client = reqwest::Client::new();
    let fitbit_client_secret = dotenv::var("FITBIT_CLIENT_SECRET")?;

    let mut request = client.post("https://api.fitbit.com/oauth2/token")
        .basic_auth(FITBIT_CLIENT_ID, Some(fitbit_client_secret))
        .form(&[("clientId", FITBIT_CLIENT_ID),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &urlencode(FITBIT_REDIRECT_URI)),
                ("code", code)])
        .send()?;
    
    let parsed: FitbitCallbackResponse = request.json()?;

    Ok(OAuthToken::from(parsed))
}

pub fn oauth_callback(service: Path<(String)>, query: Query<HashMap<String, String>>) -> HttpResponse {
    let token = match query.get("code") {
        Some(c) => oauth_flow(c),
        None => Err(OAuthError::TokenError(String::from("No token!")))
    };

    match token {
        Ok(t) => HttpResponse::Ok().json(t),
        Err(e) => HttpResponse::BadRequest().body("Bad request")
    }
}