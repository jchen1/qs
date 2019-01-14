use crate::db::{Step, Token};
use crate::utils::{urlencode};
use crate::oauth::{OAuthError, OAuthToken, OAuthProvider};
use actix_web::{error, Error};
use chrono::{offset::TimeZone, NaiveDate, NaiveDateTime, Utc, Duration};
use chrono_tz::{Tz, US::Pacific};
use uuid::Uuid;
use reqwest;

pub static FITBIT_REDIRECT_URI: &'static str = "http://localhost:8080/oauth/fitbit/callback";
pub static FITBIT_EXPIRATION_MS: i32 = 604800;

#[derive(Debug, Serialize, Deserialize)]
struct IntradayCalories {
    pub time: String,
    pub value: f32,
    pub level: i32,
    pub mets: i32
}

#[derive(Debug, Serialize, Deserialize)]
struct IntradayIntegral {
    pub time: String,
    pub value: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct IntradayFloat {
    pub time: String,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct IntradayDataset<T> {
    pub dataset: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct IntradayResponse {
    pub activities_steps_intraday: Option<IntradayDataset<IntradayIntegral>>,
    pub activities_calories_intraday: Option<IntradayDataset<IntradayCalories>>,
    pub activities_distance_intraday: Option<IntradayDataset<IntradayFloat>>,
    pub activities_floors_intraday: Option<IntradayDataset<IntradayIntegral>>,
    pub activities_elevation_intraday: Option<IntradayDataset<IntradayFloat>>,
}

fn to_step(day: NaiveDate, local_tz: Tz, step: &IntradayIntegral, user_id: Uuid) -> Step {
    // todo proper error handling
    let naive_dt = NaiveDateTime::parse_from_str(
        &format!("{}T{}", day.format("%m-%d-%Y"), step.time),
        "%m-%d-%YT%H:%M:%S",
    )
    .unwrap();
    let local_dt = local_tz.from_local_datetime(&naive_dt).unwrap();
    Step {
        time: Utc.from_utc_datetime(&local_dt.naive_utc()),
        user_id: user_id,
        source: "fitbit".to_string(),
        count: step.value,
    }
}

pub fn local_tz(_token: &Token) -> Result<Tz, Error> {
    // TODO
    Ok(Pacific)
}

pub fn steps_for_day(day: NaiveDate, token: &Token) -> Result<Vec<Step>, Error> {
    let client = reqwest::Client::new();

    let tz = local_tz(token)?;
    let endpoint = format!(
        "https://api.fitbit.com/1/user/-/activities/steps/date/{}/1d/1min/time/00:00/23:59.json",
        day.format("%Y-%m-%d")
    );

    let mut request = client
        .get(&endpoint)
        .bearer_auth(&token.access_token)
        .send()
        .map_err(error::ErrorInternalServerError)?;

    let parsed: IntradayResponse = request.json().map_err(error::ErrorInternalServerError)?;

    let steps: Vec<Step> = parsed
        .activities_steps_intraday
        .unwrap_or(IntradayDataset { dataset: vec![] })
        .dataset
        .iter()
        .map(|s| to_step(day, tz, s, token.user_id))
        .collect();

    Ok(steps)
}

pub struct Fitbit {
    oauth_id: String,
    oauth_secret: String
}

impl Fitbit {
    pub fn new(oauth_id: &str, oauth_secret: &str) -> Fitbit {
        Fitbit { oauth_id: oauth_id.to_owned(), oauth_secret: oauth_secret.to_owned() }
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
        }
    }
}

impl OAuthProvider for Fitbit {
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
