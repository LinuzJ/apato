use crate::{models::watchlist::Watchlist, oikotie::oikotie::Location};
use diesel::{prelude::*, result::Error};

use super::{establish_connection, schema::watchlists, schema::watchlists::dsl::*};
#[derive(Insertable)]
#[diesel(table_name = watchlists)]
pub struct InsertableWatchlist {
    location_id: i32,
    location_level: i32,
    location_name: String,
    user_id: i32,
    goal_yield: Option<f64>,
}

pub fn insert(location: Location, new_user_id: i32, new_goal_yield: Option<f64>) {
    let mut connection = establish_connection();

    let insertable: &InsertableWatchlist = &InsertableWatchlist {
        location_id: location.id,
        location_level: location.level,
        location_name: location.name,
        user_id: new_user_id,
        goal_yield: new_goal_yield,
    };

    match diesel::insert_into(watchlists::table)
        .values(insertable)
        .execute(&mut connection)
    {
        Ok(n) => {
            println!("Inserted {:?} rows into watchlist table", n);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

pub fn get_all() -> Vec<Watchlist> {
    let mut connection = establish_connection();

    let all_watchlists: Result<Vec<Watchlist>, diesel::result::Error> = watchlists::table
        .select(watchlists::table::all_columns())
        .get_results(&mut connection);

    match all_watchlists {
        Ok(w) => {
            println!("Watchlists: {:?}", w);
            return w;
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Vec::new();
        }
    }
}

pub fn get_for_user(id_: i32) -> Vec<Watchlist> {
    let mut connection = establish_connection();

    let r: Vec<Watchlist> = watchlists
        .filter(user_id.eq(id_))
        .select(Watchlist::as_select())
        .load(&mut connection)
        .expect("Error loading watchlists for user}");

    r
}
