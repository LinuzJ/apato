pub mod schema;

#[database("diesel_postgres_pool")]
pub struct Db(diesel::PgConnection);

use rocket_sync_db_pools::database;
