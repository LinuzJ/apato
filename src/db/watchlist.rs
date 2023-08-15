use crate::pricing::oikotie_client::Location;
use diesel::{Insertable, PgConnection, RunQueryDsl};

use super::schema::watchlists;
#[derive(Insertable)]
#[table_name = "watchlists"]
pub struct InsertableWatchlist {
    location_id: i32,
    location_level: i32,
    location_name: String,
}

pub fn insert(conn: &mut PgConnection, location: Location) {
    let insertable: &InsertableWatchlist = &InsertableWatchlist {
        location_id: location.id,
        location_level: location.level,
        location_name: location.name,
    };

    match diesel::insert_into(watchlists::table)
        .values(insertable)
        .execute(conn)
    {
        Ok(n) => println!("Inserted {:?} rows into watchlist table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}
