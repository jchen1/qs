#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::distances;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message, Object};
use crate::providers::fitbit;
use actix_web::{error, Error};

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[graphql(description = "A single distance datapoint")]
pub struct Distance {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub source: String,
    pub count: f64,
}

impl Distance {
    pub fn find_one(
        conn: &PgConnection,
        (user_id, time): (&Uuid, &DateTime<Utc>),
    ) -> Result<Distance, diesel::result::Error> {
        Ok(distances::table
            .find((user_id, time))
            .get_result::<Distance>(conn)?)
    }

    pub fn for_period(
        conn: &PgConnection,
        the_user_id: &Uuid,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<Distance>, diesel::result::Error> {
        use self::schema::distances::dsl::*;

        Ok(distances
            .filter(
                user_id
                    .eq(the_user_id)
                    .and(time.ge(start).and(time.lt(end))),
            )
            .order(time.desc())
            .load::<Distance>(conn)?)
    }
}

impl Object for Distance {
    fn insert(conn: &PgConnection, distance: &Distance) -> Result<Distance, diesel::result::Error> {
        use self::schema::distances::dsl::*;

        diesel::insert_into(distances)
            .values(distance)
            .execute(conn)?;

        Ok(Distance::find_one(
            conn,
            (&distance.user_id, &distance.time),
        )?)
    }

    // todo overload it

    fn insert_many(
        conn: &PgConnection,
        the_distances: &Vec<Distance>,
    ) -> Result<usize, diesel::result::Error> {
        use self::schema::distances::dsl::*;

        diesel::insert_into(distances)
            .values(the_distances)
            .execute(conn)
    }
}

impl fitbit::IntradayMeasurement for Distance {
    fn new(
        user_id: Uuid,
        time: DateTime<Utc>,
        measurement: fitbit::IntradayValue,
    ) -> Result<Self, Error> {
        match measurement {
            fitbit::IntradayValue::Float(count) => Ok(Distance {
                user_id: user_id,
                count: count.value,
                source: "fitbit".to_string(),
                time: time,
            }),
            _ => Err(error::ErrorInternalServerError("Wrong type!")),
        }
    }

    fn name() -> &'static str {
        "distance"
    }

    fn parse_response(r: fitbit::IntradayResponse) -> Option<Vec<fitbit::IntradayValue>> {
        r.activities_distance_intraday.and_then(|a| {
            Some(
                a.dataset
                    .into_iter()
                    .map(|v| fitbit::IntradayValue::Float(v))
                    .collect(),
            )
        })
    }
}

impl Message for Distance {
    type Result = Result<Distance, Error>;
}

impl Handler<Distance> for DbExecutor {
    type Result = Result<Distance, Error>;

    fn handle(&mut self, msg: Distance, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Distance::insert(conn, &msg)
            .map_err(|_| error::ErrorInternalServerError("Error inserting distance"))?)
    }
}
