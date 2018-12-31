use chrono::{Date, Utc};
use chrono_tz::{Tz, US::{Pacific}};
use crate::oauth::{OAuthToken};
use crate::db::{Step};
use actix_web::{Error, error};

pub fn local_tz(token: &OAuthToken) -> Result<Tz, Error> {
    // TODO
    Ok(Pacific)
}

pub fn steps_for_day(day: Date<Utc>, token: &OAuthToken) -> Result<Vec<Step>, Error> {
    let client = reqwest::Client::new();

    let tz = local_tz(token)?;
    let endpoint = format!("https://api.fitbit.com/1/user/-/activities/steps/date/{}/1d/1min/time/00:00/23:59.json",
        day);

    println!("{}", endpoint);

    let mut request = client.get(&endpoint)
        .bearer_auth(&token.access_token)
        .send()
        .map_err(|_| error::ErrorInternalServerError("failed to make request :("))?;
    
    println!("{}", request.text().map_err(|_| error::ErrorInternalServerError(""))?);

    // unimplemented!()
    Err(error::ErrorImATeapot("meh"))
}