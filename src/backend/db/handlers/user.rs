//! Db executor actor

use diesel::prelude::*;
use diesel;
use uuid;

use actix_web::{Error, error};
use crate::db::{models, schema, Message, Handler, DbExecutor};

pub struct CreateUser {
    pub email: String,
    pub g_sub: String
}

impl Message for CreateUser {
    type Result = Result<models::User, Error>;
}

impl Handler<CreateUser> for DbExecutor {
    type Result = Result<models::User, Error>;

    fn handle(&mut self, msg: CreateUser, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let uuid = uuid::Uuid::new_v4();
        let new_user = models::NewUser {
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
            .load::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}

pub struct GetUserByEmail {
    pub email: String
}

impl Message for GetUserByEmail {
    type Result = Result<models::User, Error>;
}

impl Handler<GetUserByEmail> for DbExecutor {
    type Result = Result<models::User, Error>;

    fn handle(&mut self, msg: GetUserByEmail, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;
        let conn: &PgConnection = &self.0.get().unwrap();
        let mut items = users
            .filter(email.eq(&msg.email))
            .load::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}

pub struct GetUserByGSub {
    pub g_sub: String
}

impl Message for GetUserByGSub {
    type Result = Result<models::User, Error>;
}

impl Handler<GetUserByGSub> for DbExecutor {
    type Result = Result<models::User, Error>;

    fn handle(&mut self, msg: GetUserByGSub, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;
        let conn: &PgConnection = &self.0.get().unwrap();
        let mut items = users
            .filter(g_sub.eq(&msg.g_sub))
            .load::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}

pub struct UpsertUser {
    pub email: String,
    pub g_sub: String
}

impl Message for UpsertUser {
    type Result = Result<models::User, Error>;
}

impl Handler<UpsertUser> for DbExecutor {
    type Result = Result<models::User, Error>;

    fn handle(&mut self, msg: UpsertUser, _: &mut Self::Context) -> Self::Result {
        use self::schema::users::dsl::*;

        let conn: &PgConnection = &self.0.get().unwrap();
        
        let uuid = uuid::Uuid::new_v4();
        let new_user = models::NewUser {
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
            .load::<models::User>(conn)
            .map_err(|_| error::ErrorInternalServerError("Error loading person"))?;

        Ok(items.pop().unwrap())
    }
}
