#![allow(proc_macro_derive_resolution_fallback)]

//! Db executor actor
use actix_web::{Error, error};
use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::{models, schema, Message, Handler, DbExecutor};
use self::schema::{tokens};

pub struct CreateToken {
    pub user_id: Uuid,
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

        let uuid = Uuid::new_v4();
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

pub struct UpsertToken {
    pub user_id: Uuid,
    pub service: String,
    pub service_userid: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
    pub refresh_token: String
}

impl Message for UpsertToken {
    type Result = Result<models::Token, Error>;
}

#[derive(AsChangeset)]
#[table_name="tokens"]
struct UpdateToken<'a> {
    access_token: &'a str,
    access_token_expiry: &'a DateTime<Utc>,
    service_userid: &'a str,
    refresh_token: Option<&'a str>
}

impl Handler<UpsertToken> for DbExecutor {
    type Result = Result<models::Token, Error>;

    fn handle(&mut self, msg: UpsertToken, _: &mut Self::Context) -> Self::Result {
        use self::schema::tokens::dsl::*;

        let uuid = Uuid::new_v4();
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
            .on_conflict((user_id, service))
            .do_update()
            .set(&UpdateToken {
                access_token: new_token.access_token,
                access_token_expiry: new_token.access_token_expiry,
                service_userid: new_token.service_userid,
                refresh_token: match new_token.refresh_token {
                    "" => None,
                    t => Some(t)
                }
            })
            .execute(conn)
            .map_err(|e| error::ErrorInternalServerError(format!("Error inserting token - {}", e.to_string())))?;


        let mut items = tokens
            .filter(user_id.eq(&msg.user_id).and(service.eq(&msg.service)))
            .load::<models::Token>(conn)
            .map_err(|e| error::ErrorInternalServerError(format!("Error loading token - {}", e.to_string())))?;

        Ok(items.pop().unwrap())
    }
}