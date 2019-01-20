#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::calories;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message, Object};
use crate::providers::fitbit;
use actix_web::{error, Error};

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[graphql(description = "A single calorie datapoint")]
pub struct Calorie {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub source: String,
    pub count: f64,
    pub level: i32,
    pub mets: i32,
}

impl Calorie {
    pub fn find_one(
        conn: &PgConnection,
        (user_id, time): (&Uuid, &DateTime<Utc>),
    ) -> Result<Calorie, diesel::result::Error> {
        Ok(calories::table
            .find((user_id, time))
            .get_result::<Calorie>(conn)?)
    }

    pub fn for_period(
        conn: &PgConnection,
        the_user_id: &Uuid,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<Calorie>, diesel::result::Error> {
        use self::schema::calories::dsl::*;

        Ok(calories
            .filter(
                user_id
                    .eq(the_user_id)
                    .and(time.ge(start).and(time.lt(end))),
            )
            .order(time.desc())
            .load::<Calorie>(conn)?)
    }
}

impl Object for Calorie {
    fn insert(conn: &PgConnection, calorie: &Calorie) -> Result<Calorie, diesel::result::Error> {
        use self::schema::calories::dsl::*;

        diesel::insert_into(calories)
            .values(calorie)
            .execute(conn)?;

        Ok(Calorie::find_one(conn, (&calorie.user_id, &calorie.time))?)
    }

    // todo overload it

    fn insert_many(
        conn: &PgConnection,
        the_calories: &Vec<Calorie>,
    ) -> Result<usize, diesel::result::Error> {
        use self::schema::calories::dsl::*;

        diesel::insert_into(calories)
            .values(the_calories)
            .execute(conn)
    }
}

impl fitbit::IntradayMeasurement for Calorie {
    fn new(
        user_id: Uuid,
        time: DateTime<Utc>,
        measurement: fitbit::IntradayValue,
    ) -> Result<Self, Error> {
        match measurement {
            fitbit::IntradayValue::Caloric(count) => Ok(Calorie {
                user_id: user_id,
                count: count.value,
                source: "fitbit".to_string(),
                time: time,
                level: count.level,
                mets: count.mets,
            }),
            _ => Err(error::ErrorInternalServerError("Wrong type!")),
        }
    }

    fn name() -> &'static str {
        "calories"
    }

    fn parse_response(r: fitbit::IntradayResponse) -> Option<Vec<fitbit::IntradayValue>> {
        r.activities_calories_intraday.and_then(|a| {
            Some(
                a.dataset
                    .into_iter()
                    .map(|v| fitbit::IntradayValue::Caloric(v))
                    .collect(),
            )
        })
    }
}

impl Message for Calorie {
    type Result = Result<Calorie, Error>;
}

impl Handler<Calorie> for DbExecutor {
    type Result = Result<Calorie, Error>;

    fn handle(&mut self, msg: Calorie, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Calorie::insert(conn, &msg)
            .map_err(|_| error::ErrorInternalServerError("Error inserting calorie"))?)
    }
}
