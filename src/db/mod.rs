pub mod apartment;
pub mod apartment_watchlist;
pub mod schema;
pub mod watchlist;

use std::sync::Arc;

use diesel::{Connection, PgConnection};

use crate::config::Config;

pub fn establish_connection(config: &Arc<Config>) -> PgConnection {
    let database_url = &config.db_path;

    // TODO Maybe not panic here?
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
