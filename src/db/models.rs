use super::schema::{users, tokens};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Queryable)]
pub struct User {
    pub id: Uuid,
    pub email: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: &'a Uuid,
    pub email: &'a str,
}

#[derive(Serialize, Queryable)]
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