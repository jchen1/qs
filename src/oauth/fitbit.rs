use chrono::{Utc};
use time::Duration;

pub static FITBIT_CLIENT_ID: &'static str = "22DFW9";
pub static FITBIT_REDIRECT_URI: &'static str = "http://localhost:8080/oauth/fitbit/callback";
pub static FITBIT_EXPIRATION_MS: i32 = 604800;

use super::{OAuthError, OAuthToken, urlencode};

#[derive(Deserialize)]
pub struct FitbitCallbackResponse {
    access_token: String,
    expires_in: u32,
    refresh_token: String,
    user_id: String,
    scope: String
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

pub fn redirect() -> Result<String, OAuthError> {
    let scopes = ["activity", "heartrate", "location", "profile", "sleep", "weight"].join(" ");
    Ok(format!("https://www.fitbit.com/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&expires_in={}",
        urlencode(FITBIT_CLIENT_ID),
        urlencode(FITBIT_REDIRECT_URI),
        urlencode(&scopes),
        // 1 hour
        FITBIT_EXPIRATION_MS))
}

pub fn oauth_flow(code: &str) -> Result<OAuthToken, OAuthError> {
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