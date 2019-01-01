use chrono::{Date, Utc, NaiveDate, NaiveDateTime, offset::TimeZone};
use chrono_tz::{Tz, US::{Pacific}};
use crate::db::{Step, Token};
use actix_web::{Error, error};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct StepIntradayValue {
    pub time: String,
    pub value: i32
}

#[derive(Debug, Serialize, Deserialize)]
struct StepsIntraday {
    pub dataset: Vec<StepIntradayValue>
}

#[derive(Debug, Serialize, Deserialize)]
struct StepsResponse {
    #[serde(rename="activities-steps-intraday")]
    pub activities_steps_intraday: StepsIntraday
}

fn to_step(day: NaiveDate, local_tz: Tz, step: &StepIntradayValue, user_id: Uuid) -> Step {
    // todo proper error handling
    let naive_dt = NaiveDateTime::parse_from_str(&format!("{}T{}", day.format("%m-%d-%Y"), step.time), "%m-%d-%YT%H:%M:%S").unwrap();
    let local_dt = local_tz.from_local_datetime(&naive_dt).unwrap();
    Step {
        time: Utc.from_utc_datetime(&local_dt.naive_utc()),
        user_id: user_id,
        source: "fitbit".to_string(),
        count: step.value
    }
}

pub fn local_tz(_token: &Token) -> Result<Tz, Error> {
    // TODO
    Ok(Pacific)
}

pub fn steps_for_day(day: NaiveDate, token: &Token) -> Result<Vec<Step>, Error> {
    let client = reqwest::Client::new();

    let tz = local_tz(token)?;
    let endpoint = format!("https://api.fitbit.com/1/user/-/activities/steps/date/{}/1d/1min/time/00:00/23:59.json",
        day.format("%Y-%m-%d"));

    let mut request = client.get(&endpoint)
        .bearer_auth(&token.access_token)
        .send()
        .map_err(|_| error::ErrorInternalServerError("failed to make request :("))?;
    
    let parsed: StepsResponse = request.json()
        .map_err(|_| error::ErrorInternalServerError("couldn't deserialize steps json :("))?;

    let steps: Vec<Step> = parsed.activities_steps_intraday.dataset.iter().map(|s| to_step(day, tz, s, token.user_id)).collect();

    Ok(steps)
}