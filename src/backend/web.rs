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
mod worker;

use actix::prelude::*;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::{fs::NamedFile, http::Method, middleware, server, App, Error, State};
use listenfd::ListenFd;
use std::thread;
use std::sync::{Arc, RwLock};
use std::time::{Duration};

pub struct AppState {
    db: Addr<db::DbExecutor>,
    graphql: Addr<graphql::GraphQLExecutor>
}

fn index(_state: State<AppState>) -> Result<NamedFile, Error> {
    Ok(NamedFile::open("src/index.html")?)
}

fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let db_url = dotenv::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:password@localhost/dev_db".to_string());
    let redis_url = dotenv::var("REDIS_URL").unwrap_or("redis://localhost".to_string());
    let cookie_key = dotenv::var("JWT_ISSUE").unwrap_or(" ".repeat(32).to_string());
    let queue_name = dotenv::var("WORKER_QUEUE_NAME").unwrap_or("default".to_string());
    let env = dotenv::var("ENVIORNMENT").unwrap_or("dev".to_string());

    let sys = actix::System::new("qs");

    let pool = db::init_pool(db_url);
    let graphql_pool = pool.clone();

    let num_workers: u32 = dotenv::var("NUM_WORKERS")
        .unwrap_or("1".to_string())
        .parse()
        .unwrap();
    let mut threads = vec![];
    let worker_redis_url = redis_url.clone();
    let worker_queue_name = queue_name.clone();
    let worker_pool = pool.clone();

    let is_running = Arc::new(RwLock::new(true));

    let db_addr = SyncArbiter::start(3, move || db::DbExecutor(pool.clone()));

    let schema = std::sync::Arc::new(graphql::schema::create_schema());
    let graphql_addr = SyncArbiter::start(3, move || {
        graphql::GraphQLExecutor::new(
            schema.clone(),
            graphql_pool.clone(),
            queue::init_queue(redis_url.clone(), queue_name.clone()),
        )
    });

    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(move || {
        App::with_state(AppState {
            db: db_addr.clone(),
            graphql: graphql_addr.clone(),
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

    if env == "dev" {
        for i in 0..num_workers {
            let redis_url = worker_redis_url.clone();
            let queue_name = worker_queue_name.clone();
            let conn = worker_pool.get().unwrap();
            let is_running = is_running.clone();

            threads.push(thread::spawn(move || {
                info!("Started thread {}", i);
                let queue = queue::init_queue(redis_url.clone(), queue_name.clone());
                let ctx = worker::WorkerContext {
                    queue: queue,
                    conn: db::Conn(conn),
                };

                loop {
                    let is_running = *is_running.read().unwrap();
                    if is_running {
                        match worker::pop_and_execute(&ctx) {
                            // YOLO
                            Ok(Some(_)) => (),
                            Ok(None) => {
                                thread::sleep(Duration::from_secs(5))
                            }
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
