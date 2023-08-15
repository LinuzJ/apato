use crate::{models::watchlist::Watchlist, oikotie::oikotie_client::Location};
use diesel::{query_dsl::select_dsl::SelectDsl, Insertable, PgConnection, RunQueryDsl, Table};

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

pub fn get_all(conn: &mut PgConnection, watchlist_id: i32) -> Vec<Watchlist> {
    let all_watchlists = watchlists::table
        .select(watchlists::table::all_columns())
        .load(conn);

    match all_watchlists {
        Ok(watchlists) => {
            println!("Watchlists: {:?}", watchlists);
            return watchlists;
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Vec::new();
        }
    }
}
