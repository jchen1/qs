use juniper::{FieldResult, RootNode};
use uuid::Uuid;
use diesel::prelude::*;
use std::ops::Deref;

use crate::oauth::{self, OAuthToken};
use super::Context;
use crate::db;
use chrono::{DateTime, Utc};

#[derive(GraphQLInputObject)]
#[graphql(description = "A user")]
struct NewUser {
    id: Uuid,
    email: String,
    g_sub: String
}

#[derive(GraphQLObject)]
#[graphql(description = "A token")]
struct Token {
    pub id: Uuid,
    pub service: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>
}

impl From<&db::Token> for Token {
    fn from(token: &db::Token) -> Self {
        Token {
            id: token.id,
            service: token.service.clone(),
            access_token: token.access_token.clone(),
            access_token_expiry: token.access_token_expiry
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "A user")]
struct User {
    pub id: Uuid,
    pub email: String,
    pub g_sub: String,
    pub tokens: Vec<Token>
}

impl User {
    pub fn new(user: db::User, tokens: Vec<db::Token>) -> User {
        User {
            id: user.id,
            email: user.email,
            g_sub: user.g_sub,
            tokens: tokens.iter().map(|t| Token::from(t)).collect()
        }
    }
}

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {

    field user(&executor, id: Option<String>) -> FieldResult<Option<User>> {
        let conn = &executor.context().conn;
        let user = match id {
            Some(id) => db::User::find_one(conn, &Uuid::parse_str(&id)?).ok(),
            None => executor.context().user.clone()
        };
        let tokens = user.clone().and_then(|u| db::Token::belonging_to(&u).load::<db::Token>(conn.deref()).ok());

        Ok(user.map(|u| User::new(u, tokens.unwrap_or(vec![]))))
    }

    field OAuthServiceURL(&executor, service: String) -> FieldResult<String> {
        let uri = oauth::start_oauth(service);

        match uri {
            Ok(uri) => Ok(uri),
            Err(_e) => Err("Service unimplemented".to_owned())?
        }
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field create_user(&executor, new_user: NewUser) -> FieldResult<User> {
        Ok(User{
            id: Uuid::new_v4(),
            email: new_user.email,
            g_sub: new_user.g_sub,
            tokens: vec![]
        })
    }

    field refresh_token(&executor, token_id: Uuid) -> FieldResult<Token> {
        let conn = &executor.context().conn;

        let token = db::Token::find_one(conn, token_id)?;
        let new_token = oauth::refresh_token(OAuthToken::from(token))?;
        let updated = db::Token::update(conn, token_id, db::UpdateToken {
            access_token: Some(&new_token.access_token),
            access_token_expiry: Some(&new_token.expiration),
            service_userid: Some(&new_token.user_id),
            refresh_token: match new_token.refresh_token.as_str() {
                "" => None,
                e => Some(&e)
            }
        })?;
        Ok(Token::from(&updated))
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}