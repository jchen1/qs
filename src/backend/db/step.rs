#![allow(proc_macro_derive_resolution_fallback)]

use diesel::pg::PgConnection;
use diesel::prelude::*;
use super::schema::{steps};
use uuid::Uuid;
use chrono::{Utc, DateTime};

use actix_web::{error, Error};
use crate::db::{schema, Message, Handler, DbExecutor};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct Step {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub source: String,
    pub count: i32
}

#[derive(Insertable)]
#[table_name = "steps"]
pub struct NewStep<'a> {
    pub time: &'a DateTime<Utc>,
    pub user_id: &'a Uuid,
    pub source: &'a str,
    pub count: &'a i32
}

impl Step {
    pub fn find_one(conn: &PgConnection, (user_id, time): (&Uuid, &DateTime<Utc>)) -> Result<Step, diesel::result::Error> {
        Ok(steps::table.find((user_id, time)).get_result::<Step>(conn)?)
    }

    pub fn for_period(conn: &PgConnection, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<Step>, diesel::result::Error> {
        unimplemented!()
    }

    pub fn insert(conn: &PgConnection, step: NewStep) -> Result<Step, diesel::result::Error> {
        use self::schema::steps::dsl::*;

        diesel::insert_into(steps)
            .values(&step)
            .execute(conn)?;

        Ok(Step::find_one(conn, (&step.user_id, &step.time))?)
    }
}

impl<'a> Message for NewStep<'a> {
    type Result = Result<Step, Error>;
}

impl<'a> Handler<NewStep<'a>> for DbExecutor {
    type Result = Result<Step, Error>;

    fn handle(&mut self, msg: NewStep, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Step::insert(conn, msg).map_err(|_| error::ErrorInternalServerError("Error inserting step"))?)
    }
}
