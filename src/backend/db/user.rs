#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::users;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::{schema, DbExecutor, Handler, Message};
use actix_web::{error, Error};

#[derive(Identifiable, Debug, Clone, Serialize, Queryable)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub g_sub: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: &'a Uuid,
    pub email: &'a str,
    pub g_sub: &'a str,
}

impl User {
    pub fn insert(conn: &PgConnection, user: &NewUser) -> Result<User, diesel::result::Error> {
        use self::schema::users::dsl::*;

        diesel::insert_into(users).values(user).execute(conn)?;

        Ok(User::find_one(conn, &user.id)?)
    }

    pub fn upsert(conn: &PgConnection, user: &NewUser) -> Result<User, diesel::result::Error> {
        use self::schema::users::dsl::*;

        diesel::insert_into(users)
            .values(user)
            .on_conflict_do_nothing()
            .execute(conn)?;

        Ok(User::find_one_by_email(conn, &user.email)?)
    }

    pub fn find_one(conn: &PgConnection, id: &Uuid) -> Result<User, diesel::result::Error> {
        Ok(users::table.find(id).get_result::<User>(conn)?)
    }

    pub fn find_one_by_email(
        conn: &PgConnection,
        the_email: &str,
    ) -> Result<User, diesel::result::Error> {
        use self::schema::users::dsl::*;
        let mut items = users.filter(email.eq(the_email)).load::<User>(conn)?;

        Ok(items.pop().unwrap())
    }

    pub fn find_one_by_g_sub(
        conn: &PgConnection,
        the_gsub: &str,
    ) -> Result<User, diesel::result::Error> {
        use self::schema::users::dsl::*;
        let mut items = users.filter(g_sub.eq(the_gsub)).load::<User>(conn)?;

        Ok(items.pop().unwrap())
    }
}

pub struct CreateUser {
    pub email: String,
    pub g_sub: String,
}

impl Message for CreateUser {
    type Result = Result<User, Error>;
}

impl Handler<CreateUser> for DbExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        let uuid = Uuid::new_v4();
        let new_user = NewUser {
            id: &uuid,
            email: &msg.email,
            g_sub: &msg.g_sub,
        };

        let conn: &PgConnection = &self.0.get().unwrap();

        Ok(User::insert(conn, &new_user)
            .map_err(|_| error::ErrorInternalServerError("Error inserting user!"))?)
    }
}

pub struct GetUserByEmail {
    pub email: String,
}

impl Message for GetUserByEmail {
    type Result = Result<User, Error>;
}

impl Handler<GetUserByEmail> for DbExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, msg: GetUserByEmail, _: &mut Self::Context) -> Self::Result {
        let conn: &PgConnection = &self.0.get().unwrap();
        Ok(User::find_one_by_email(conn, &msg.email)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?)
    }
}

pub struct GetUserByGSub {
    pub g_sub: String,
}

impl Message for GetUserByGSub {
    type Result = Result<User, Error>;
}

impl Handler<GetUserByGSub> for DbExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, msg: GetUserByGSub, _: &mut Self::Context) -> Self::Result {
        let conn: &PgConnection = &self.0.get().unwrap();
        Ok(User::find_one_by_g_sub(conn, &msg.g_sub)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?)
    }
}

pub struct UpsertUser {
    pub email: String,
    pub g_sub: String,
}

impl Message for UpsertUser {
    type Result = Result<User, Error>;
}

impl Handler<UpsertUser> for DbExecutor {
    type Result = Result<User, Error>;

    fn handle(&mut self, msg: UpsertUser, _: &mut Self::Context) -> Self::Result {
        let conn: &PgConnection = &self.0.get().unwrap();

        let uuid = Uuid::new_v4();
        let new_user = NewUser {
            id: &uuid,
            email: &msg.email,
            g_sub: &msg.g_sub,
        };

        Ok(User::upsert(conn, &new_user)
            .map_err(|_| error::ErrorInternalServerError("Error upserting user"))?)
    }
}
