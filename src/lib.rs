#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models; 
pub mod telemetry;
pub mod routes;
pub mod domain;
pub mod email;

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::{Pool, PooledConnection};
pub type DBPool = Pool<ConnectionManager<PgConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;
