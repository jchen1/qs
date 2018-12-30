use juniper::{FieldResult, RootNode};
use uuid::Uuid;

use crate::oauth;
use super::Context;
use crate::db::models::{User};

#[derive(GraphQLInputObject)]
#[graphql(description = "A user")]
struct NewUser {
    id: Uuid,
    email: String,
    g_sub: String
}

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {

    field user(&executor, id: Option<String>) -> FieldResult<Option<User>> {
        match id {
            Some(id) => Ok(User::find_one(&executor.context().conn, Uuid::parse_str(&id)?).ok()),
            None => Ok(executor.context().user.clone())
        }
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
            g_sub: new_user.g_sub
        })
    }

    field FinishOAuthServiceFlow(&executor, service: String, code: String) -> FieldResult<User> {
        let token = match service.as_str() {
            "fitbit" => oauth::fitbit::oauth_flow(&code),
            "google" => oauth::google::oauth_flow(&code),
            _ => Err(oauth::OAuthError::Error(String::from("Bad service")))
        };
        unimplemented!()
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}