use crate::oauth::{OAuthError, OAuthProvider, OAuthToken};
use crate::utils::urlencode;
use crate::db::{Token};
use actix_web::{Error};
use chrono::{Duration, Utc};
use chrono_tz::{Tz, US::Pacific};
use reqwest;

pub static FITBIT_REDIRECT_URI: &'static str = "http://localhost:8080/oauth/fitbit/callback";
pub static FITBIT_EXPIRATION_MS: i32 = 604800;

pub mod intraday;
pub use crate::providers::fitbit::intraday::*;

pub struct Fitbit {
    oauth_id: String,
    oauth_secret: String,
}

impl Fitbit {
    pub fn new(oauth_id: &str, oauth_secret: &str) -> Fitbit {
        Fitbit {
            oauth_id: oauth_id.to_owned(),
            oauth_secret: oauth_secret.to_owned(),
        }
    }
}

#[derive(Deserialize)]
pub struct FitbitCallbackResponse {
    access_token: String,
    expires_in: u32,
    refresh_token: String,
    user_id: String,
    scope: String,
}

impl From<FitbitCallbackResponse> for OAuthToken {
    fn from(fcr: FitbitCallbackResponse) -> Self {
        OAuthToken {
            service: "fitbit".to_string(),
            access_token: fcr.access_token,
            refresh_token: fcr.refresh_token,
            user_id: fcr.user_id,
            scopes: fcr.scope.split(" ").map(String::from).collect(),
            email: None,
            expiration: Utc::now() + Duration::seconds(fcr.expires_in as i64),
            g_sub: None
        }
    }
}

impl OAuthProvider for Fitbit {
    fn name(&self) -> &'static str {
        "fitbit"
    }

    fn oauth_redirect_url(&self) -> Result<String, OAuthError> {
        let scopes = [
            "activity",
            "heartrate",
            "location",
            "profile",
            "sleep",
            "weight",
        ]
        .join(" ");
        Ok(format!("https://www.fitbit.com/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&expires_in={}",
            urlencode(&self.oauth_id),
            urlencode(FITBIT_REDIRECT_URI),
            urlencode(&scopes),
            // 1 hour
            FITBIT_EXPIRATION_MS))
    }

    fn token_from_code(&self, code: &str) -> Result<OAuthToken, OAuthError> {
        let client = reqwest::Client::new();
        let mut request = client
            .post("https://api.fitbit.com/oauth2/token")
            .basic_auth(&self.oauth_id, Some(&self.oauth_secret))
            .form(&[
                ("clientId", self.oauth_id.as_str()),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &urlencode(FITBIT_REDIRECT_URI)),
                ("code", code),
            ])
            .send()?;

        let parsed: FitbitCallbackResponse = request.json()?;
        Ok(OAuthToken::from(parsed))
    }

    fn refresh_token(&self, token: OAuthToken) -> Result<OAuthToken, OAuthError> {
        let client = reqwest::Client::new();
        let mut request = client
            .post("https://api.fitbit.com/oauth2/token")
            .basic_auth(&self.oauth_id, Some(&self.oauth_secret))
            .form(&[
                ("clientId", self.oauth_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", &token.refresh_token),
            ])
            .send()?;

        let parsed: FitbitCallbackResponse = request.json()?;
        Ok(OAuthToken::from(parsed))
    }
}

pub fn local_tz(_token: &Token) -> Result<Tz, Error> {
    // TODO
    Ok(Pacific)
}
