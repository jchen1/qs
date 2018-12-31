#![allow(proc_macro_derive_resolution_fallback)]

use diesel::pg::PgConnection;
use diesel::prelude::*;
use super::schema::{users};
use uuid::Uuid;

use actix_web::{Error, error};
use crate::db::{self, schema, Message, Handler, DbExecutor};


#[derive(Identifiable, Debug, Clone, Serialize, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub g_sub: String
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: &'a Uuid,
    pub email: &'a str,
    pub g_sub: &'a str
}

impl User {
    pub fn find_one(conn: &PgConnection, id: Uuid) -> Result<User, diesel::result::Error> {
        Ok(users::table.find(id).get_result::<User>(conn)?)
    }
}

pub struct CreateUser {
    pub email: String,
    pub g_sub: String
}

impl Message for CreateUser {
    type Result = Result<db::User, Error>;
}

impl Handler<CreateUser> for DbExecutor {
    type Result = Result<db::User, Error>;

    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let uuid = Uuid::new_v4();
        let new_user = db::NewUser {
            id: &uuid,
            email: &msg.email,
            g_sub: &msg.g_sub
        };

        let conn: &PgConnection = &self.0.get().unwrap();

        diesel::insert_into(users)
            .values(&new_user)
            .execute(conn)
            .map_err(|_| error::ErrorInternalServerError("Error inserting person"))?;

        let mut items = users
            .filter(id.eq(&uuid))
            .load::<db::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}

pub struct GetUserByEmail {
    pub email: String
}

impl Message for GetUserByEmail {
    type Result = Result<db::User, Error>;
}

impl Handler<GetUserByEmail> for DbExecutor {
    type Result = Result<db::User, Error>;

    fn handle(&mut self, msg: GetUserByEmail, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;
        let conn: &PgConnection = &self.0.get().unwrap();
        let mut items = users
            .filter(email.eq(&msg.email))
            .load::<db::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}

pub struct GetUserByGSub {
    pub g_sub: String
}

impl Message for GetUserByGSub {
    type Result = Result<db::User, Error>;
}

impl Handler<GetUserByGSub> for DbExecutor {
    type Result = Result<db::User, Error>;

    fn handle(&mut self, msg: GetUserByGSub, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;
        let conn: &PgConnection = &self.0.get().unwrap();
        let mut items = users
            .filter(g_sub.eq(&msg.g_sub))
            .load::<db::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}

pub struct UpsertUser {
    pub email: String,
    pub g_sub: String
}

impl Message for UpsertUser {
    type Result = Result<db::User, Error>;
}

impl Handler<UpsertUser> for DbExecutor {
    type Result = Result<db::User, Error>;

    fn handle(&mut self, msg: UpsertUser, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let conn: &PgConnection = &self.0.get().unwrap();
        
        let uuid = Uuid::new_v4();
        let new_user = db::NewUser {
            id: &uuid,
            email: &msg.email,
            g_sub: &msg.g_sub
        };

        diesel::insert_into(users)
            .values(&new_user)
            .on_conflict_do_nothing()
            .execute(conn)
            .map_err(|_| error::ErrorInternalServerError("Error inserting person"))?;

        let mut items = users
            .filter(g_sub.eq(&msg.g_sub))
            .load::<db::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}
