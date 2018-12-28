#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

extern crate actix;
extern crate actix_web;
extern crate listenfd;
extern crate futures;
extern crate reqwest;
extern crate url;
extern crate hyper;
extern crate base64;
extern crate uuid;
extern crate r2d2;

pub mod oauth;
pub mod db;

use listenfd::ListenFd;
use actix::prelude::*;
use actix_web::{http::Method, middleware, server, App, HttpRequest, State};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

/// State with DbExecutor address
pub struct AppState {
    db: Addr<db::DbExecutor>,
}

fn index(state: State<AppState>) -> &'static str {
    "Hello world!"
}

fn main() {
    dotenv::dotenv().ok();
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

    let addr = SyncArbiter::start(3, move || db::DbExecutor(pool.clone()));

    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
        App::with_state(AppState{db: addr.clone()})
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.method(Method::GET).with(index))
            .resource("/oauth/{service}/start", |r| r.method(Method::GET).with(oauth::oauth_start))
            .resource("/oauth/{service}/callback", |r| r.method(Method::GET).with(oauth::oauth_callback))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run();
    sys.run();
}