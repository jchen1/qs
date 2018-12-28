//! Db executor actor
use crate::actix::prelude::*;
use actix_web::*;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid;
use chrono::{DateTime, Utc};

mod models;
pub mod schema;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct CreateUser {
    pub email: String,
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

pub struct CreateToken {
    pub user_id: uuid::Uuid,
    pub service: String,
    pub service_userid: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
    pub refresh_token: String
}

impl Message for CreateToken {
    type Result = Result<models::Token, Error>;
}

impl Handler<CreateToken> for DbExecutor {
    type Result = Result<models::Token, Error>;

    fn handle(&mut self, msg: CreateToken, _: &mut Self::Context) -> Self::Result {
        use self::schema::tokens::dsl::*;

        let uuid = uuid::Uuid::new_v4();
        let new_token = models::NewToken {
            id: &uuid,
            user_id: &msg.user_id,
            service: &msg.service,
            service_userid: &msg.service_userid,
            access_token: &msg.access_token,
            access_token_expiry: &msg.access_token_expiry,
            refresh_token: &msg.refresh_token
        };

        let conn: &PgConnection = &self.0.get().unwrap();

        diesel::insert_into(tokens)
            .values(&new_token)
            .execute(conn)
            .map_err(|e| error::ErrorInternalServerError(format!("Error inserting token - {}", e.to_string())))?;


        let mut items = tokens
            .filter(id.eq(&uuid))
            .load::<models::Token>(conn)
            .map_err(|e| error::ErrorInternalServerError(format!("Error loading token - {}", e.to_string())))?;

        Ok(items.pop().unwrap())
    }
}