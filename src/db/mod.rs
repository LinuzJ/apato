pub mod apartment;
pub mod schema;
pub mod watchlist;

use dotenvy::dotenv;
use std::{
    env,
    ops::{Deref, DerefMut},
};

#[database("diesel_postgres_pool")]
pub struct Db(diesel::PgConnection);

impl Deref for Db {
    type Target = diesel::PgConnection;

    fn deref(&self) -> &Self::Target {
        &self
    }
}

impl DerefMut for Db {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

use diesel::{Connection, PgConnection};
use rocket_sync_db_pools::database;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
