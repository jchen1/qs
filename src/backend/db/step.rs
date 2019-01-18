#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::steps;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message};
use crate::providers::fitbit;
use actix_web::{error, Error};

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[graphql(description = "A single step datapoint")]
pub struct Step {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub source: String,
    pub count: i32,
}

impl Step {
    pub fn find_one(
        conn: &PgConnection,
        (user_id, time): (&Uuid, &DateTime<Utc>),
    ) -> Result<Step, diesel::result::Error> {
        Ok(steps::table
            .find((user_id, time))
            .get_result::<Step>(conn)?)
    }

    pub fn for_period(
        conn: &PgConnection,
        the_user_id: &Uuid,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<Step>, diesel::result::Error> {
        use self::schema::steps::dsl::*;

        Ok(steps
            .filter(
                user_id
                    .eq(the_user_id)
                    .and(time.ge(start).and(time.lt(end))),
            )
            .order(time.desc())
            .load::<Step>(conn)?)
    }

    pub fn insert(conn: &PgConnection, step: &Step) -> Result<Step, diesel::result::Error> {
        use self::schema::steps::dsl::*;

        diesel::insert_into(steps).values(step).execute(conn)?;

        Ok(Step::find_one(conn, (&step.user_id, &step.time))?)
    }

    // todo overload it

    pub fn insert_many(
        conn: &PgConnection,
        the_steps: &Vec<Step>,
    ) -> Result<usize, diesel::result::Error> {
        use self::schema::steps::dsl::*;

        diesel::insert_into(steps).values(the_steps).execute(conn)
    }
}

impl fitbit::IntradayMeasurement for Step {
    fn new(user_id: Uuid, time: DateTime<Utc>, measurement: fitbit::IntradayValue) -> Result<Self, Error> {
        match measurement {
            fitbit::IntradayValue::Integral(count) => {
                Ok(Step {
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
        "step"
    }

    fn parse_response(r: fitbit::IntradayResponse) -> Option<Vec<fitbit::IntradayValue>> {
        r.activities_steps_intraday.and_then(|a| {
            Some(
                a.dataset
                    .into_iter()
                    .map(|v| fitbit::IntradayValue::Integral(v))
                    .collect(),
            )
        })
    }
}

impl Message for Step {
    type Result = Result<Step, Error>;
}

impl Handler<Step> for DbExecutor {
    type Result = Result<Step, Error>;

    fn handle(&mut self, msg: Step, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Step::insert(conn, &msg)
            .map_err(|_| error::ErrorInternalServerError("Error inserting step"))?)
    }
}
