use actix::prelude::{Actor, Addr, Handler, Message, SyncContext};
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{
    error, http::header, AsyncResponder, FromRequest, FutureResponse, HttpRequest, HttpResponse,
    Path, Query,
};
use chrono::{DateTime, Utc};
use futures::{
    future::{err, ok, result},
    Future,
};
use std::collections::HashMap;
use std::fmt::{self, Display};
use uuid::Uuid;

use super::AppState;
use crate::db::{self, DbExecutor, UpsertToken, UpsertUser};

#[derive(Serialize, Deserialize)]
pub struct OAuthToken {
    pub service: String,
    pub access_token: String,
    pub expiration: DateTime<Utc>,
    pub refresh_token: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub email: Option<String>,
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
            email: None,
        }
    }
}

#[derive(Debug)]
pub enum OAuthError {
    DotEnv(dotenv::Error),
    Reqwest(reqwest::Error),
    ActixError(actix_web::error::Error),
    TokenError(String),
    Error(String),
}

impl Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OAuthError::DotEnv(e) => write!(f, "{}", e),
            OAuthError::Reqwest(e) => write!(f, "{}", e),
            OAuthError::TokenError(e) => write!(f, "{}", e),
            OAuthError::Error(e) => write!(f, "{}", e),
            OAuthError::ActixError(e) => write!(f, "{}", e),
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

impl From<actix_web::error::Error> for OAuthError {
    fn from(e: actix_web::error::Error) -> Self {
        OAuthError::ActixError(e)
    }
}

pub trait OAuthProvider {
    fn oauth_redirect_url(&self) -> Result<String, OAuthError>;
    fn token_from_code(&self, code: &str) -> Result<OAuthToken, OAuthError>;
    fn refresh_token(&self, token: OAuthToken) -> Result<OAuthToken, OAuthError>;
}

// TODO
pub struct OAuth {
    providers: HashMap<String, Box<OAuthProvider + Send + Sync>>,
}

impl OAuth {
    pub fn new(providers: HashMap<String, Box<OAuthProvider + Send + Sync>>) -> Self {
        OAuth {
            providers: providers,
        }
    }

    pub fn redirect_url(&self, service: &str) -> Result<String, OAuthError> {
        let provider = self
            .providers
            .get(&service.to_string())
            .ok_or(OAuthError::Error("Service not implemented".to_string()))?;
        provider.oauth_redirect_url()
    }

    pub fn callback(&self, service: &str, code: &str) -> Result<OAuthToken, OAuthError> {
        let provider = self
            .providers
            .get(&service.to_string())
            .ok_or(OAuthError::Error("Service not implemented".to_string()))?;
        provider.token_from_code(code)
    }

    pub fn refresh_token(&self, token: OAuthToken) -> Result<OAuthToken, OAuthError> {
        let provider = self
            .providers
            .get(&token.service)
            .ok_or(OAuthError::Error("Service not implemented".to_string()))?;
        provider.refresh_token(token)
    }
}

pub struct OAuthExecutor(pub OAuth);

impl Actor for OAuthExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Serialize, Deserialize)]
pub struct OAuthRequest(String);

impl Message for OAuthRequest {
    type Result = Result<String, OAuthError>;
}

impl Handler<OAuthRequest> for OAuthExecutor {
    type Result = Result<String, OAuthError>;
    fn handle(&mut self, msg: OAuthRequest, _: &mut Self::Context) -> Self::Result {
        let oauth = &self.0;
        oauth.redirect_url(msg.0.as_str())
    }
}

#[derive(Serialize, Deserialize)]
pub struct OAuthCallback {
    pub service: String,
    pub code: String,
}

impl Message for OAuthCallback {
    type Result = Result<OAuthToken, OAuthError>;
}

impl Handler<OAuthCallback> for OAuthExecutor {
    type Result = Result<OAuthToken, OAuthError>;
    fn handle(&mut self, msg: OAuthCallback, _: &mut Self::Context) -> Self::Result {
        let oauth = &self.0;
        oauth.callback(msg.service.as_str(), msg.code.as_str())
    }
}

#[derive(Serialize, Deserialize)]
pub struct OAuthRefresh(OAuthToken);

impl Message for OAuthRefresh {
    type Result = Result<OAuthToken, OAuthError>;
}

impl Handler<OAuthRefresh> for OAuthExecutor {
    type Result = Result<OAuthToken, OAuthError>;
    fn handle(&mut self, msg: OAuthRefresh, _: &mut Self::Context) -> Self::Result {
        let oauth = &self.0;
        oauth.refresh_token(msg.0)
    }
}

pub fn start_oauth(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let service =
        Path::<String>::extract(&req).unwrap_or(Path::<String>::from("not-a-service".to_owned()));
    let oauth = &req.state().oauth;

    oauth
        .send(OAuthRequest(service.to_string()))
        .from_err()
        .and_then(|res| match res {
            Ok(url) => Ok(HttpResponse::Found().header(header::LOCATION, url).finish()),
            Err(e) => {
                error!("{}", e);
                Err(error::ErrorBadRequest("Bad request"))
            }
        })
        .responder()
}

fn try_login(
    db: &Addr<DbExecutor>,
    maybe_userid: &Option<String>,
    token: &OAuthToken,
) -> FutureResponse<String> {
    if let Some(id) = maybe_userid {
        info!("Already logged in as: {}", id);
        return Box::new(result::<String, actix_web::Error>(Ok(id.to_string())));
    } else if token.service == "google" {
        db.send(UpsertUser {
            email: match &token.email {
                Some(email) => email,
                None => unimplemented!(),
            }
            .clone(),
            g_sub: token.user_id.clone(),
        })
        .from_err()
        .and_then(|res| match res {
            Ok(user) => {
                info!("Logged in as {} ({})", user.id, user.email);
                Ok(user.id.to_string())
            }
            Err(_e) => unimplemented!(),
        })
        .responder()
    } else {
        unimplemented!()
    }
}

pub fn oauth_callback(
    req: &HttpRequest<AppState>,
) -> Result<FutureResponse<HttpResponse>, actix_web::Error> {
    let req = req.clone();
    let db = req.state().db.clone();
    let oauth = &req.state().oauth;

    let service = Path::<String>::extract(&req)?.into_inner();
    let query = Query::<HashMap<String, String>>::extract(&req)?;
    let code = query
        .get("code")
        .ok_or(error::ErrorBadRequest("Bad request"))?
        .to_string();
    let params = OAuthCallback {
        service: service,
        code: code,
    };

    Ok(oauth
        .send(params)
        .from_err()
        .and_then(|maybe_token| match maybe_token {
            Ok(token) => ok(token),
            Err(e) => err(error::ErrorInternalServerError(e)),
        })
        .and_then(move |t| {
            try_login(&db, &req.identity(), &t)
                .and_then(move |user_id| {
                    req.remember(user_id.clone());
                    db.send(UpsertToken {
                        access_token: t.access_token,
                        access_token_expiry: t.expiration,
                        refresh_token: t.refresh_token,
                        service: t.service,
                        service_userid: t.user_id,
                        user_id: Uuid::parse_str(&user_id).unwrap(),
                    })
                    .from_err()
                })
                .and_then(|res| match res {
                    Ok(_) => Ok(HttpResponse::Found().header(header::LOCATION, "/").finish()),
                    Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
                })
                .responder()
        })
        .responder())
}

pub fn logout(req: &HttpRequest<AppState>) -> HttpResponse {
    req.forget();
    HttpResponse::Found().header(header::LOCATION, "/").finish()
}
