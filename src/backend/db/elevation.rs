#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::elevations;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message};
use crate::providers::fitbit;
use actix_web::{error, Error};

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[graphql(description = "A single elevation datapoint")]
pub struct Elevation {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub source: String,
    pub count: f64,
}

impl Elevation {
    pub fn find_one(
        conn: &PgConnection,
        (user_id, time): (&Uuid, &DateTime<Utc>),
    ) -> Result<Elevation, diesel::result::Error> {
        Ok(elevations::table
            .find((user_id, time))
            .get_result::<Elevation>(conn)?)
    }

    pub fn for_period(
        conn: &PgConnection,
        the_user_id: &Uuid,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<Elevation>, diesel::result::Error> {
        use self::schema::elevations::dsl::*;

        Ok(elevations
            .filter(
                user_id
                    .eq(the_user_id)
                    .and(time.ge(start).and(time.lt(end))),
            )
            .order(time.desc())
            .load::<Elevation>(conn)?)
    }

    pub fn insert(conn: &PgConnection, elevation: &Elevation) -> Result<Elevation, diesel::result::Error> {
        use self::schema::elevations::dsl::*;

        diesel::insert_into(elevations).values(elevation).execute(conn)?;

        Ok(Elevation::find_one(conn, (&elevation.user_id, &elevation.time))?)
    }

    // todo overload it

    pub fn insert_many(
        conn: &PgConnection,
        the_elevations: &Vec<Elevation>,
    ) -> Result<usize, diesel::result::Error> {
        use self::schema::elevations::dsl::*;

        diesel::insert_into(elevations).values(the_elevations).execute(conn)
    }
}

impl fitbit::IntradayMeasurement for Elevation {
    fn new(user_id: Uuid, time: DateTime<Utc>, measurement: fitbit::IntradayValue) -> Result<Self, Error> {
        match measurement {
            fitbit::IntradayValue::Float(count) => {
                Ok(Elevation {
                    user_id: user_id,
                    count: count.value,
                    source: "fitbit".to_string(),
                    time: time,
                })
            },
            _ => Err(error::ErrorInternalServerError("Wrong type!"))
        }
    }

    fn name() -> &'static str {
        "elevation"
    }

    fn parse_response(r: fitbit::IntradayResponse) -> Option<Vec<fitbit::IntradayValue>> {
        r.activities_elevation_intraday.and_then(|a| {
            Some(
                a.dataset
                    .into_iter()
                    .map(|v| fitbit::IntradayValue::Float(v))
                    .collect(),
            )
        })
    }
}

impl Message for Elevation {
    type Result = Result<Elevation, Error>;
}

impl Handler<Elevation> for DbExecutor {
    type Result = Result<Elevation, Error>;

    fn handle(&mut self, msg: Elevation, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Elevation::insert(conn, &msg)
            .map_err(|_| error::ErrorInternalServerError("Error inserting elevation"))?)
    }
}
