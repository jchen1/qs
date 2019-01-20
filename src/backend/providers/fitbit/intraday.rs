use super::local_tz;
use crate::db::Token;
use actix_web::{error, Error};
use chrono::{offset::TimeZone, DateTime, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Tz;
use reqwest;
use uuid::Uuid;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IntradayMetric {
    Step,
    Calorie,
    Distance,
    Elevation,
    Floor,
}

pub trait IntradayMeasurement: Sized + Debug {
    fn new(
        user_id: uuid::Uuid,
        time: DateTime<Utc>,
        measurement: IntradayValue,
    ) -> Result<Self, Error>;
    fn parse_response(r: IntradayResponse) -> Option<Vec<IntradayValue>>;
    fn name() -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntradayCalories {
    pub time: String,
    pub value: f64,
    pub level: i32,
    pub mets: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntradayIntegral {
    pub time: String,
    pub value: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntradayFloat {
    pub time: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IntradayValue {
    Integral(IntradayIntegral),
    Float(IntradayFloat),
    Caloric(IntradayCalories),
}

impl IntradayValue {
    fn time_str(&self) -> &str {
        match self {
            IntradayValue::Integral(v) => &v.time,
            IntradayValue::Float(v) => &v.time,
            IntradayValue::Caloric(v) => &v.time,
        }
    }

    fn time_utc(&self, day: NaiveDate, local_tz: Tz) -> Result<DateTime<Utc>, Error> {
        let naive_dt = NaiveDateTime::parse_from_str(
            &format!("{}T{}", day.format("%m-%d-%Y"), self.time_str()),
            "%m-%d-%YT%H:%M:%S",
        )
        .map_err(error::ErrorInternalServerError)?;
        let local_dt = local_tz.from_local_datetime(&naive_dt).earliest().ok_or(
            error::ErrorInternalServerError("error converting timestamp"),
        )?;

        Ok(Utc.from_utc_datetime(&local_dt.naive_utc()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntradayDataset<T> {
    pub dataset: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IntradayResponse {
    pub activities_steps_intraday: Option<IntradayDataset<IntradayIntegral>>,
    pub activities_calories_intraday: Option<IntradayDataset<IntradayCalories>>,
    pub activities_distance_intraday: Option<IntradayDataset<IntradayFloat>>,
    pub activities_floors_intraday: Option<IntradayDataset<IntradayIntegral>>,
    pub activities_elevation_intraday: Option<IntradayDataset<IntradayFloat>>,
}

fn to_measurement<T: IntradayMeasurement>(
    day: NaiveDate,
    local_tz: Tz,
    measurement: IntradayValue,
    user_id: Uuid,
) -> Result<T, Error> {
    T::new(user_id, measurement.time_utc(day, local_tz)?, measurement)
}

pub fn measurement_for_day<T: IntradayMeasurement>(
    day: NaiveDate,
    token: &Token,
) -> Result<Vec<T>, Error> {
    let client = reqwest::Client::new();

    let tz = local_tz(token)?;
    let endpoint = format!(
        "https://api.fitbit.com/1/user/-/activities/{}/date/{}/1d/1min/time/00:00/23:59.json",
        T::name(),
        day.format("%Y-%m-%d")
    );

    let mut request = client
        .get(&endpoint)
        .bearer_auth(&token.access_token)
        .send()
        .map_err(error::ErrorInternalServerError)?;

    let resp: IntradayResponse = request.json().map_err(error::ErrorInternalServerError)?;

    let measurements: Vec<T> = T::parse_response(resp)
        .unwrap_or(vec![])
        .into_iter()
        .map(|s| to_measurement(day, tz, s, token.user_id))
        .filter(|s| s.is_ok())
        .map(|s| s.expect("uh oh"))
        .collect();

    Ok(measurements)
}
