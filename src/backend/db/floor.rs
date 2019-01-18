#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::floors;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message};
use crate::providers::fitbit;
use actix_web::{error, Error};

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[graphql(description = "A single floor datapoint")]
pub struct Floor {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub source: String,
    pub count: i32,
}

impl Floor {
    pub fn find_one(
        conn: &PgConnection,
        (user_id, time): (&Uuid, &DateTime<Utc>),
    ) -> Result<Floor, diesel::result::Error> {
        Ok(floors::table
            .find((user_id, time))
            .get_result::<Floor>(conn)?)
    }

    pub fn for_period(
        conn: &PgConnection,
        the_user_id: &Uuid,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<Floor>, diesel::result::Error> {
        use self::schema::floors::dsl::*;

        Ok(floors
            .filter(
                user_id
                    .eq(the_user_id)
                    .and(time.ge(start).and(time.lt(end))),
            )
            .order(time.desc())
            .load::<Floor>(conn)?)
    }

    pub fn insert(conn: &PgConnection, floor: &Floor) -> Result<Floor, diesel::result::Error> {
        use self::schema::floors::dsl::*;

        diesel::insert_into(floors).values(floor).execute(conn)?;

        Ok(Floor::find_one(conn, (&floor.user_id, &floor.time))?)
    }

    // todo overload it

    pub fn insert_many(
        conn: &PgConnection,
        the_floors: &Vec<Floor>,
    ) -> Result<usize, diesel::result::Error> {
        use self::schema::floors::dsl::*;

        diesel::insert_into(floors).values(the_floors).execute(conn)
    }
}

impl fitbit::IntradayMeasurement for Floor {
    fn new(user_id: Uuid, time: DateTime<Utc>, measurement: fitbit::IntradayValue) -> Result<Self, Error> {
        match measurement {
            fitbit::IntradayValue::Integral(count) => {
                Ok(Floor {
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
        "floor"
    }

    fn parse_response(r: fitbit::IntradayResponse) -> Option<Vec<fitbit::IntradayValue>> {
        r.activities_floors_intraday.and_then(|a| {
            Some(
                a.dataset
                    .into_iter()
                    .map(|v| fitbit::IntradayValue::Integral(v))
                    .collect(),
            )
        })
    }
}

impl Message for Floor {
    type Result = Result<Floor, Error>;
}

impl Handler<Floor> for DbExecutor {
    type Result = Result<Floor, Error>;

    fn handle(&mut self, msg: Floor, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Floor::insert(conn, &msg)
            .map_err(|_| error::ErrorInternalServerError("Error inserting floor"))?)
    }
}
