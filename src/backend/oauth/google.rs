extern crate jsonwebtoken;

use chrono::Utc;
use jsonwebtoken::{dangerous_unsafe_decode, TokenData};
use time::Duration;
use uuid::Uuid;

pub static GOOGLE_CLIENT_ID: &'static str =
    "820579007787-k29hdg84c8170kp4k60jdgj2soncluau.apps.googleusercontent.com";
pub static GOOGLE_REDIRECT_URI: &'static str = "http://localhost:8080/oauth/google/callback";

use super::{OAuthError, OAuthToken};
use crate::utils::{urlencode};

#[derive(Deserialize)]
pub struct GoogleCallbackResponse {
    access_token: String,
    id_token: String,
    expires_in: u32,
    refresh_token: Option<String>,
    scope: String,
}

#[derive(Deserialize)]
struct GoogleProfileClaims {
    // TODO: use the access token hash to verify
    // at_hash: String,
    email: String,
    sub: String,
}

impl From<GoogleCallbackResponse> for OAuthToken {
    fn from(gcr: GoogleCallbackResponse) -> Self {
        let TokenData { claims, .. } =
            dangerous_unsafe_decode::<GoogleProfileClaims>(&gcr.id_token)
                .expect("Bad Google response!");

        OAuthToken {
            service: "google".to_string(),
            access_token: gcr.access_token,
            refresh_token: match gcr.refresh_token {
                Some(token) => token,
                // TODO throw an error? idk
                None => "".to_string(),
            },
            user_id: claims.sub,
            email: Some(claims.email),
            scopes: gcr.scope.split(" ").map(String::from).collect(),
            expiration: Utc::now() + Duration::seconds(gcr.expires_in as i64),
        }
    }
}

#[derive(Deserialize)]
pub struct EndpointInfo {
    authorization_endpoint: String,
    token_endpoint: String,
    // userinfo_endpoint: String,
    // revocation_endpoint: String,
    // jwks_uri: String
}

fn get_discovery_doc() -> Result<EndpointInfo, OAuthError> {
    let client = reqwest::Client::new();
    let res: EndpointInfo = client
        .get("https://accounts.google.com/.well-known/openid-configuration")
        .send()?
        .json()?;
    return Ok(res);
}

pub fn redirect() -> Result<String, OAuthError> {
    let scopes = ["openid", "email"].join(" ");
    // todo a real state/session cookie
    let state = format!("{}", Uuid::new_v4());
    let authorization_endpoint = get_discovery_doc()?.authorization_endpoint;
    Ok(format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&access_type=offline",
        authorization_endpoint,
        urlencode(GOOGLE_CLIENT_ID),
        urlencode(GOOGLE_REDIRECT_URI),
        urlencode(&scopes),
        urlencode(state.as_str())
    ))
}

pub fn oauth_flow(code: &str) -> Result<OAuthToken, OAuthError> {
    dotenv::dotenv().ok();

    let client = reqwest::Client::new();
    let google_client_secret = dotenv::var("GOOGLE_CLIENT_SECRET")?;
    let token_endpoint = get_discovery_doc()?.token_endpoint;

    let mut request = client
        .post(&token_endpoint)
        .form(&[
            ("client_id", GOOGLE_CLIENT_ID),
            ("client_secret", &google_client_secret),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &urlencode(GOOGLE_REDIRECT_URI)),
            ("code", code),
        ])
        .send()?;

    let parsed: GoogleCallbackResponse = request.json()?;
    let token = OAuthToken::from(parsed);

    Ok(token)
}

pub fn refresh(token: OAuthToken) -> Result<OAuthToken, OAuthError> {
    let client = reqwest::Client::new();
    let google_client_secret = dotenv::var("GOOGLE_CLIENT_SECRET")?;
    let token_endpoint = get_discovery_doc()?.token_endpoint;

    let mut request = client
        .post(&token_endpoint)
        .form(&[
            ("client_id", GOOGLE_CLIENT_ID),
            ("client_secret", &google_client_secret),
            ("grant_type", "refresh_token"),
            ("refresh_token", &token.refresh_token),
        ])
        .send()?;

    let parsed: GoogleCallbackResponse = request.json()?;

    Ok(OAuthToken::from(parsed))
}
