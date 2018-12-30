use juniper::{FieldResult, RootNode};
use uuid::Uuid;

use crate::oauth;

#[derive(GraphQLObject)]
#[graphql(description = "A user")]
struct User {
    id: Uuid,
    email: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A user")]
struct NewUser {
    id: Uuid,
    email: String,
}

pub struct QueryRoot;

graphql_object!(QueryRoot: () |&self| {
    field user(&executor, id: String) -> User {
        User{
            id: Uuid::new_v4(),
            email: "hello@jeff.yt".to_owned()
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

graphql_object!(MutationRoot: () |&self| {
    field create_user(&executor, new_user: NewUser) -> FieldResult<User> {
        Ok(User{
            id: Uuid::new_v4(),
            email: new_user.email
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