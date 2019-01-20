//! Db executor actor
use actix::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::ops::Deref;

pub mod calorie;
pub mod distance;
pub mod elevation;
pub mod floor;
pub mod step;
pub use crate::db::calorie::*;
pub use crate::db::distance::*;
pub use crate::db::elevation::*;
pub use crate::db::floor::*;
pub use crate::db::step::*;

pub mod token;
pub mod user;
pub use crate::db::token::*;
pub use crate::db::user::*;

pub mod schema;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub trait Object: Sized {
    fn insert(conn: &PgConnection, obj: &Self) -> Result<Self, diesel::result::Error>;
    fn insert_many(conn: &PgConnection, objs: &Vec<Self>) -> Result<usize, diesel::result::Error>;
}

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
