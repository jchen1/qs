use actix::prelude::*;
use actix_web::{AsyncResponder, FutureResponse, Error, HttpRequest, HttpResponse, Json, State};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use super::AppState;
use futures::future::Future;

pub mod schema;

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: std::sync::Arc<schema::Schema>,
}

impl GraphQLExecutor {
    pub fn new(schema: std::sync::Arc<schema::Schema>) -> GraphQLExecutor {
        GraphQLExecutor { schema: schema }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &());
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

pub fn graphql(
    (st, data): (State<AppState>, Json<GraphQLData>),
) -> FutureResponse<HttpResponse> {
    st.graphql
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(data) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(data)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}