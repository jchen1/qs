#![allow(proc_macro_derive_resolution_fallback)]

use super::schema::tokens;
use super::user::User;
use crate::db::{self, schema, DbExecutor, Handler, Message};
use actix_web::{error, Error};
use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Identifiable, Associations, Debug, Clone, Serialize, Queryable)]
#[belongs_to(User)]
#[table_name = "tokens"]
pub struct Token {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service: String,
    pub service_userid: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
    pub refresh_token: String,
}

#[derive(Insertable)]
#[table_name = "tokens"]
pub struct NewToken<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
    pub service: &'a str,
    pub service_userid: &'a str,
    pub access_token: &'a str,
    pub access_token_expiry: &'a DateTime<Utc>,
    pub refresh_token: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "tokens"]
pub struct UpdateToken<'a> {
    pub access_token: Option<&'a str>,
    pub access_token_expiry: Option<&'a DateTime<Utc>>,
    pub service_userid: Option<&'a str>,
    pub refresh_token: Option<&'a str>,
}

impl Token {
    pub fn find_one(conn: &PgConnection, id: Uuid) -> Result<Token, diesel::result::Error> {
        Ok(tokens::table.find(id).get_result::<Token>(conn)?)
    }

    pub fn find_by_uid_service(
        conn: &PgConnection,
        the_user_id: &Uuid,
        the_service: &str,
    ) -> Result<Token, diesel::result::Error> {
        use self::schema::tokens::dsl::*;

        let mut items = tokens
            .filter(user_id.eq(the_user_id).and(service.eq(the_service)))
            .load::<Token>(conn)?;

        // todo assert at most one
        Ok(items.pop().unwrap())
    }

    pub fn update(
        conn: &PgConnection,
        id: Uuid,
        update: UpdateToken,
    ) -> Result<Token, diesel::result::Error> {
        let _ = diesel::update(tokens::table.find(id))
            .set(&update)
            .execute(conn)?;
        Token::find_one(conn, id)
    }
}

pub struct CreateToken {
    pub user_id: Uuid,
    pub service: String,
    pub service_userid: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
    pub refresh_token: String,
}

impl Message for CreateToken {
    type Result = Result<db::Token, Error>;
}

impl Handler<CreateToken> for DbExecutor {
    type Result = Result<db::Token, Error>;

    fn handle(&mut self, msg: CreateToken, _: &mut Self::Context) -> Self::Result {
        use self::schema::tokens::dsl::*;

        let uuid = Uuid::new_v4();
        let new_token = db::NewToken {
            id: &uuid,
            user_id: &msg.user_id,
            service: &msg.service,
            service_userid: &msg.service_userid,
            access_token: &msg.access_token,
            access_token_expiry: &msg.access_token_expiry,
            refresh_token: &msg.refresh_token,
        };

        let conn: &PgConnection = &self.0.get().unwrap();

        diesel::insert_into(tokens)
            .values(&new_token)
            .execute(conn)
            .map_err(|e| {
                error::ErrorInternalServerError(format!(
                    "Error inserting token - {}",
                    e.to_string()
                ))
            })?;

        let mut items = tokens
            .filter(id.eq(&uuid))
            .load::<db::Token>(conn)
            .map_err(|e| {
                error::ErrorInternalServerError(format!("Error loading token - {}", e.to_string()))
            })?;

        Ok(items.pop().unwrap())
    }
}

pub struct UpsertToken {
    pub user_id: Uuid,
    pub service: String,
    pub service_userid: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
    pub refresh_token: String,
}

impl Message for UpsertToken {
    type Result = Result<db::Token, Error>;
}

impl Handler<UpsertToken> for DbExecutor {
    type Result = Result<db::Token, Error>;

    fn handle(&mut self, msg: UpsertToken, _: &mut Self::Context) -> Self::Result {
        use self::schema::tokens::dsl::*;

        let uuid = Uuid::new_v4();
        let new_token = db::NewToken {
            id: &uuid,
            user_id: &msg.user_id,
            service: &msg.service,
            service_userid: &msg.service_userid,
            access_token: &msg.access_token,
            access_token_expiry: &msg.access_token_expiry,
            refresh_token: &msg.refresh_token,
        };

        let conn: &PgConnection = &self.0.get().unwrap();

        diesel::insert_into(tokens)
            .values(&new_token)
            .on_conflict((user_id, service))
            .do_update()
            .set(&UpdateToken {
                access_token: Some(new_token.access_token),
                access_token_expiry: Some(new_token.access_token_expiry),
                service_userid: Some(new_token.service_userid),
                refresh_token: match new_token.refresh_token {
                    "" => None,
                    t => Some(t),
                },
            })
            .execute(conn)
            .map_err(|e| {
                error::ErrorInternalServerError(format!(
                    "Error inserting token - {}",
                    e.to_string()
                ))
            })?;

        let mut items = tokens
            .filter(user_id.eq(&msg.user_id).and(service.eq(&msg.service)))
            .load::<db::Token>(conn)
            .map_err(|e| {
                error::ErrorInternalServerError(format!("Error loading token - {}", e.to_string()))
            })?;

        Ok(items.pop().unwrap())
    }
}
