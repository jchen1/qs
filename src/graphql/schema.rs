use juniper::FieldResult;
use juniper::RootNode;
use uuid::Uuid;

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
    field user(&executor, id: String) -> FieldResult<User> {
        Ok(User{
            id: Uuid::new_v4(),
            email: "hello@jeff.yt".to_owned()
        })
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: () |&self| {
    field createUser(&executor, new_user: NewUser) -> FieldResult<User> {
        Ok(User{
            id: Uuid::new_v4(),
            email: new_user.email
        })
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}