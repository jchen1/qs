//! Db executor actor
use crate::actix::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, self};
use uuid;
use std::ops::Deref;

pub mod models;
pub mod schema;
mod handlers;

pub use crate::db::handlers::*;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub fn init_pool(database_url: String) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(manager).expect("Failed to create pool.")
}

pub struct Conn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Conn {
    type Target = PgConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}