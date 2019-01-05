use actix::prelude::*;
use crate::{
    db::{self, User},
    AppState,
    queue::{Queue}
};
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{AsyncResponder, Error, FutureResponse, HttpMessage, HttpRequest, HttpResponse};
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::Context as JuniperContext;
use uuid::Uuid;

pub mod schema;

#[derive(Serialize, Deserialize)]
pub struct GraphQLData {
    req: GraphQLRequest,
    user_id: Option<Uuid>,
}

pub struct Context {
    pub conn: db::Conn,
    pub user: Option<User>,
    pub producer: Queue
}

impl JuniperContext for Context {}

impl Context {
    pub fn new(conn: db::Conn, user: Option<User>, producer: Queue) -> Context {
        Context {
            conn: conn,
            user: user,
            producer: producer
        }
    }
}

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: std::sync::Arc<schema::Schema>,
    pool: db::Pool,
    producer: Queue
}

impl GraphQLExecutor {
    pub fn new(schema: std::sync::Arc<schema::Schema>, pool: db::Pool, producer: Queue) -> GraphQLExecutor {
        GraphQLExecutor {
            schema: schema,
            pool: pool,
            producer: producer
        }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let conn = self.pool.get().unwrap();
        let user = msg.user_id.and_then(|id| User::find_one(&conn, &id).ok());
        let context = Context::new(db::Conn(conn), user, self.producer.clone());

        let res = msg.req.execute(&self.schema, &context);
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

pub fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = graphiql_source("http://localhost:8080/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub fn graphql(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let req = req.clone();
    let executor = req.state().graphql.clone();
    let user_id = req.identity().and_then(|id| Uuid::parse_str(&id).ok());

    req.json()
        .from_err()
        .and_then(move |req: GraphQLRequest| {
            executor
                .send(GraphQLData {
                    req: req,
                    user_id: user_id,
                })
                .from_err()
                .and_then(|res| match res {
                    Ok(data) => Ok(HttpResponse::Ok().body(data)),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}
