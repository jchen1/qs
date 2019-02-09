#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::moods;
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message, Object};
use actix_web::{error, Error};

#[derive(GraphQLObject, Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[graphql(description = "A single mood datapoint")]
pub struct Mood {
    pub time: DateTime<Utc>,
    pub user_id: Uuid,
    pub mood: i32,
    pub note: String,
}

impl Mood {
    pub fn for_period(
        conn: &PgConnection,
        the_user_id: &Uuid,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<Mood>, diesel::result::Error> {
        use self::schema::moods::dsl::*;

        Ok(moods
            .filter(
                user_id
                    .eq(the_user_id)
                    .and(time.ge(start).and(time.lt(end))),
            )
            .order(time.desc())
            .load::<Mood>(conn)?)
    }

    pub fn find_one(
        conn: &PgConnection,
        (user_id, time): &(Uuid, DateTime<Utc>),
    ) -> Result<Mood, diesel::result::Error> {
        Ok(moods::table
            .find((user_id, time))
            .get_result::<Mood>(conn)?)
    }
}

impl Object for Mood {
    fn insert(conn: &PgConnection, the_mood: &Mood) -> Result<Mood, diesel::result::Error> {
        use self::schema::moods::dsl::*;

        diesel::insert_into(moods).values(the_mood).execute(conn)?;

        Ok(Mood::find_one(conn, &(the_mood.user_id, the_mood.time))?)
    }

    // todo overload it

    fn insert_many(
        conn: &PgConnection,
        the_moods: &[Mood],
    ) -> Result<usize, diesel::result::Error> {
        use self::schema::moods::dsl::*;

        diesel::insert_into(moods).values(the_moods).execute(conn)
    }
}

impl Message for Mood {
    type Result = Result<Mood, Error>;
}

impl Handler<Mood> for DbExecutor {
    type Result = Result<Mood, Error>;

    fn handle(&mut self, msg: Mood, _: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        Ok(Mood::insert(conn, &msg)
            .map_err(|_| error::ErrorInternalServerError("Error inserting mood"))?)
    }
}
