use jsonwebtoken::{dangerous_unsafe_decode, TokenData};
use time::Duration;
use uuid::Uuid;

use crate::oauth::{OAuthError, OAuthProvider, OAuthToken};
use crate::utils::urlencode;

use chrono::Utc;
use reqwest;

pub static GOOGLE_REDIRECT_URI: &'static str = "http://localhost:8080/oauth/google/callback";

pub struct Google {
    oauth_id: String,
    oauth_secret: String,
}

impl Google {
    pub fn new(oauth_id: &str, oauth_secret: &str) -> Google {
        Google {
            oauth_id: oauth_id.to_owned(),
            oauth_secret: oauth_secret.to_owned(),
        }
    }
}

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
            user_id: claims.sub.clone(),
            g_sub: Some(claims.sub.clone()),
            email: Some(claims.email),
            scopes: gcr.scope.split(' ').map(String::from).collect(),
            expiration: Utc::now() + Duration::seconds(i64::from(gcr.expires_in)),
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
    Ok(res)
}

impl OAuthProvider for Google {
    fn name(&self) -> &'static str {
        "google"
    }

    fn oauth_redirect_url(&self) -> Result<String, OAuthError> {
        let scopes = ["openid", "email"].join(" ");
        // todo a real state/session cookie
        let state = format!("{}", Uuid::new_v4());
        let authorization_endpoint = get_discovery_doc()?.authorization_endpoint;
        Ok(format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&access_type=offline",
            authorization_endpoint,
            urlencode(&self.oauth_id),
            urlencode(GOOGLE_REDIRECT_URI),
            urlencode(&scopes),
            urlencode(state.as_str())
        ))
    }

    fn token_from_code(&self, code: &str) -> Result<OAuthToken, OAuthError> {
        let client = reqwest::Client::new();
        let token_endpoint = get_discovery_doc()?.token_endpoint;

        let mut request = client
            .post(&token_endpoint)
            .form(&[
                ("client_id", self.oauth_id.as_str()),
                ("client_secret", self.oauth_secret.as_str()),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &urlencode(GOOGLE_REDIRECT_URI)),
                ("code", code),
            ])
            .send()?;

        let parsed: GoogleCallbackResponse = request.json()?;
        let token = OAuthToken::from(parsed);

        Ok(token)
    }

    fn refresh_token(&self, token: OAuthToken) -> Result<OAuthToken, OAuthError> {
        let client = reqwest::Client::new();
        let token_endpoint = get_discovery_doc()?.token_endpoint;

        let mut request = client
            .post(&token_endpoint)
            .form(&[
                ("client_id", self.oauth_id.as_str()),
                ("client_secret", self.oauth_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", &token.refresh_token),
            ])
            .send()?;

        let parsed: GoogleCallbackResponse = request.json()?;

        Ok(OAuthToken::from(parsed))
    }
}
