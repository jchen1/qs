#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate log;

pub mod db;
pub mod graphql;
mod middlewares;
pub mod oauth;
pub mod providers;
pub mod queue;
pub mod utils;
mod worker;

use crate::oauth::OAuthProvider;
use crate::providers::{fitbit::Fitbit, google::Google};
use actix::prelude::*;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::{http::Method, middleware, server, App};
use listenfd::ListenFd;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct AppState {
    db: Addr<db::DbExecutor>,
    graphql: Addr<graphql::GraphQLExecutor>,
    oauth: Addr<oauth::OAuthExecutor>,
}

fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // env vars with dev defaults
    let db_url = dotenv::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/dev_db".to_string());
    let redis_url = dotenv::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost".to_string());
    let cookie_key = dotenv::var("JWT_ISSUE").unwrap_or_else(|_| " ".repeat(32).to_string());
    let queue_name = dotenv::var("WORKER_QUEUE_NAME").unwrap_or_else(|_| "default".to_string());
    let env = dotenv::var("ENVIORNMENT").unwrap_or_else(|_| "dev".to_string());
    let num_workers: u32 = dotenv::var("NUM_WORKERS")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .unwrap();

    let fitbit_id = dotenv::var("FITBIT_CLIENT_ID").unwrap_or_else(|_| "22DFW9".to_string());
    let google_id = dotenv::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_|
        "820579007787-k29hdg84c8170kp4k60jdgj2soncluau.apps.googleusercontent.com".to_string(),
    );

    // env vars that crash the system
    let fitbit_secret = dotenv::var("FITBIT_CLIENT_SECRET").unwrap();
    let google_secret = dotenv::var("GOOGLE_CLIENT_SECRET").unwrap();

    let sys = actix::System::new("qs");

    let pool = db::init_pool(db_url);
    let graphql_pool = pool.clone();

    let mut threads = vec![];
    let worker_redis_url = redis_url.clone();
    let worker_queue_name = queue_name.clone();
    let worker_pool = pool.clone();

    let is_running = Arc::new(RwLock::new(true));

    let db_addr = SyncArbiter::start(3, move || db::DbExecutor(pool.clone()));

    let schema = std::sync::Arc::new(graphql::schema::create_schema());
    let graphql_addr = SyncArbiter::start(2, move || {
        graphql::GraphQLExecutor::new(
            schema.clone(),
            graphql_pool.clone(),
            queue::init_queue(&redis_url, queue_name.clone()),
        )
    });

    let fitbit_id_clone = fitbit_id.clone();
    let fitbit_secret_clone = fitbit_secret.clone();
    let google_id_clone = google_id.clone();
    let google_secret_clone = google_secret.clone();

    let oauth_addr = SyncArbiter::start(2, move || {
        let mut oauth_providers: HashMap<String, Box<OAuthProvider + Send + Sync>> = HashMap::new();
        oauth_providers.insert(
            "fitbit".to_string(),
            Box::new(Fitbit::new(&fitbit_id, &fitbit_secret)),
        );
        oauth_providers.insert(
            "google".to_string(),
            Box::new(Google::new(&google_id, &google_secret)),
        );

        oauth::OAuthExecutor(oauth::OAuth::new(oauth_providers))
    });

    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
        App::with_state(AppState {
            db: db_addr.clone(),
            graphql: graphql_addr.clone(),
            oauth: oauth_addr.clone(),
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
        .resource("/oauth/{service}/start", |r| {
            r.method(Method::GET).f(oauth::start_oauth)
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

    if env == "dev" {
        for i in 0..num_workers {
            let redis_url = worker_redis_url.clone();
            let queue_name = worker_queue_name.clone();
            let conn = worker_pool.get().unwrap();
            let is_running = is_running.clone();

            let fitbit_id_clone = fitbit_id_clone.clone();
            let fitbit_secret_clone = fitbit_secret_clone.clone();
            let google_id_clone = google_id_clone.clone();
            let google_secret_clone = google_secret_clone.clone();

            threads.push(thread::spawn(move || {
                info!("Started thread {}", i);

                let mut oauth_providers: HashMap<String, Box<OAuthProvider + Send + Sync>> =
                    HashMap::new();
                oauth_providers.insert(
                    "fitbit".to_string(),
                    Box::new(Fitbit::new(
                        &fitbit_id_clone.clone(),
                        &fitbit_secret_clone.clone(),
                    )),
                );
                oauth_providers.insert(
                    "google".to_string(),
                    Box::new(Google::new(
                        &google_id_clone.clone(),
                        &google_secret_clone.clone(),
                    )),
                );
                let queue = queue::init_queue(&redis_url, queue_name.clone());
                let ctx = worker::WorkerContext {
                    queue,
                    conn: db::Conn(conn),
                    oauth: oauth::OAuth::new(oauth_providers),
                };

                loop {
                    let is_running = *is_running.read().unwrap();
                    if is_running {
                        match worker::pop_and_execute(&ctx) {
                            // YOLO
                            Ok(Some(_)) => (),
                            Ok(None) => thread::sleep(Duration::from_secs(5)),
                            Err(_) => (),
                        }
                    } else {
                        break;
                    }
                }
            }));
        }
    }

    server.run();
    sys.run();

    {
        let mut w = is_running.write().unwrap();
        *w = false;
    }

    info!("Shutting down...");

    for thread in threads {
        let _ = thread.join();
    }
    info!("Worker threads shut down.");
}
