#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate log;

extern crate actix;
extern crate actix_web;
extern crate base64;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate listenfd;
extern crate oppgave;
extern crate r2d2;
extern crate reqwest;
extern crate url;
extern crate uuid;

pub mod db;
pub mod graphql;
mod middlewares;
pub mod oauth;
pub mod providers;
mod queue;

use actix::prelude::*;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::{fs::NamedFile, Error, http::Method, middleware, server, App, State};
use listenfd::ListenFd;

/// State with DbExecutor address
pub struct AppState {
    db: Addr<db::DbExecutor>,
    graphql: Addr<graphql::GraphQLExecutor>,
    // redis: Addr<queue::QueueExecutor>
}

fn index(_state: State<AppState>) -> Result<NamedFile, Error> {
    Ok(NamedFile::open("src/index.html")?)
}

fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let db_url = dotenv::var("DATABASE_URL").unwrap_or("postgres://postgres:password@localhost/dev_db".to_string());
    let redis_url = dotenv::var("REDIS_URL").unwrap_or("redis://localhost".to_string());
    let cookie_key = dotenv::var("JWT_ISSUE").unwrap_or(" ".repeat(32).to_string());
    let queue_name = dotenv::var("WORKER_QUEUE_NAME").unwrap_or("default".to_string());

    let sys = actix::System::new("qs");

    let pool = db::init_pool(db_url);
    let graphql_pool = pool.clone();

    let db_addr = SyncArbiter::start(3, move || db::DbExecutor(pool.clone()));

    let schema = std::sync::Arc::new(graphql::schema::create_schema());
    let graphql_addr = SyncArbiter::start(3, move || {
        graphql::GraphQLExecutor::new(schema.clone(),
                                      graphql_pool.clone(),
                                      queue::init_queue(redis_url.clone(), queue_name.clone()))
    });

    // let redis_addr = SyncArbiter::start(3, move || queue::QueueExecutor(queue_clone));

    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
        App::with_state(AppState {
            db: db_addr.clone(),
            graphql: graphql_addr.clone(),
            // redis: redis_addr.clone()
        })
        .middleware(middleware::Logger::default())
        // TODO secure: true on prod
        .middleware(SessionStorage::new(
            CookieSessionBackend::signed(cookie_key.as_bytes())
                .secure(false)
                .http_only(true),
        ))
        .middleware(IdentityService::new(
            CookieIdentityPolicy::new(cookie_key.as_bytes())
                .name("auth")
                .secure(false),
        ))
        .resource("/", |r| r.method(Method::GET).with(index))
        .resource("/oauth/{service}/start", |r| {
            r.method(Method::GET).f(oauth::start_oauth_route)
        })
        .resource("/oauth/{service}/callback", |r| {
            r.method(Method::GET).f(oauth::oauth_callback)
        })
        .resource("/graphql", |r| r.method(Method::POST).f(graphql::graphql))
        .resource("/graphiql", |r| r.method(Method::GET).h(graphql::graphiql))
        .resource("/logout", |r| r.method(Method::GET).f(oauth::logout))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8081").unwrap()
    };

    server.run();
    sys.run();
}
