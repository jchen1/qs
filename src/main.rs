#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use] extern crate juniper;

extern crate actix;
extern crate actix_web;
extern crate listenfd;
extern crate env_logger;
extern crate futures;
extern crate reqwest;
extern crate url;
extern crate hyper;
extern crate base64;
extern crate uuid;
extern crate r2d2;

pub mod oauth;
pub mod db;
pub mod graphql;

use listenfd::ListenFd;
use actix::prelude::*;
use actix_web::{http::Method, middleware, fs::NamedFile, server, App, State};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

/// State with DbExecutor address
pub struct AppState {
    db: Addr<db::DbExecutor>,
    graphql: Addr<graphql::GraphQLExecutor>
}

fn index(_state: State<AppState>) -> Result<NamedFile, actix_web::Error> {
    Ok(NamedFile::open("src/index.html")?)
}

fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let db_url = match dotenv::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_e) => unimplemented!()
    };

    let sys = actix::System::new("qs");

    // Start 3 db executor actors
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let db_addr = SyncArbiter::start(3, move || db::DbExecutor(pool.clone()));

    let schema = std::sync::Arc::new(graphql::schema::create_schema());
    let graphql_addr = SyncArbiter::start(3, move || graphql::GraphQLExecutor::new(schema.clone()));

    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
        App::with_state(AppState{db: db_addr.clone(), graphql: graphql_addr.clone()})
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.method(Method::GET).with(index))
            .resource("/oauth/{service}/start", |r| r.method(Method::GET).with(oauth::oauth_start))
            .resource("/oauth/{service}/callback", |r| r.method(Method::GET).with(oauth::oauth_callback))
            .resource("/graphql", |r| r.method(Method::POST).with(graphql::graphql))
            .resource("/graphiql", |r| r.method(Method::GET).h(graphql::graphiql))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run();
    sys.run();
}