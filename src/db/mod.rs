pub mod apartment;
pub mod schema;
pub mod watchlist;

use dotenvy::dotenv;
use std::env;

use diesel::{Connection, PgConnection};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // TODO Maybe not panic here?
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
