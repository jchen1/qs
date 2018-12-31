use juniper::{FieldResult, RootNode};
use uuid::Uuid;
use diesel::prelude::*;
use std::ops::Deref;

use crate::oauth;
use super::Context;
use crate::db::models;
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

impl From<&models::Token> for Token {
    fn from(token: &models::Token) -> Self {
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
    pub fn new(user: models::User, tokens: Vec<models::Token>) -> User {
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
            Some(id) => models::User::find_one(conn, Uuid::parse_str(&id)?).ok(),
            None => executor.context().user.clone()
        };
        let tokens = user.clone().and_then(|u| models::Token::belonging_to(&u).load::<models::Token>(conn.deref()).ok());

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
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}