use crate::db::{Step, Token};
use actix_web::{error, Error};
use chrono::{offset::TimeZone, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::{Tz, US::Pacific};
use uuid::Uuid;

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
