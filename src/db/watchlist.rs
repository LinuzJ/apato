use crate::{models::watchlist::Watchlist, oikotie::oikotie::Location};
use diesel::{
    query_dsl::select_dsl::SelectDsl, result::Error, Insertable, PgConnection, RunQueryDsl, Table,
};

use super::{establish_connection, schema::watchlists};
#[derive(Insertable)]
#[table_name = "watchlists"]
pub struct InsertableWatchlist {
    location_id: i32,
    location_level: i32,
    location_name: String,
}

pub fn create(location: Location) {
    let mut connection = establish_connection();

    // Insert
    let _ = insert(&mut connection, location);
}

pub fn get_all() -> Vec<Watchlist> {
    let mut connection = establish_connection();

    return get_all_watchlists(&mut connection);
}

fn insert(conn: &mut PgConnection, location: Location) -> Result<usize, Error> {
    let insertable: &InsertableWatchlist = &InsertableWatchlist {
        location_id: location.id,
        location_level: location.level,
        location_name: location.name,
    };

    match diesel::insert_into(watchlists::table)
        .values(insertable)
        .execute(conn)
    {
        Ok(n) => {
            println!("Inserted {:?} rows into watchlist table", n);
            return Ok(n);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(e);
        }
    }
}

fn get_all_watchlists(conn: &mut PgConnection) -> Vec<Watchlist> {
    let all_watchlists: Result<Vec<Watchlist>, diesel::result::Error> = watchlists::table
        .select(watchlists::table::all_columns())
        .get_results(conn);

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
