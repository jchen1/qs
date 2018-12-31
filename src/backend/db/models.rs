#![allow(proc_macro_derive_resolution_fallback)]

use diesel::pg::PgConnection;
use diesel::prelude::*;
use super::schema::{users, tokens};
use chrono::{DateTime, Utc};
use uuid::Uuid;

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
    pub refresh_token: String
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
    pub refresh_token: &'a str
}