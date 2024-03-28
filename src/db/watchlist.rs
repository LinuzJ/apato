use crate::models::watchlist::InsertableWatchlist;
use crate::{models::watchlist::Watchlist, oikotie::oikotie::Location};
use diesel::prelude::*;
use log::error;
use log::info;

use super::{establish_connection, schema::watchlists, schema::watchlists::dsl::*};

pub fn insert(location: Location, new_user_id: i32, new_goal_yield: Option<f64>) {
    let mut connection = establish_connection();

    let watchlist: InsertableWatchlist = InsertableWatchlist {
        location_id: location.id,
        location_level: location.level,
        location_name: location.name,
        user_id: new_user_id,
        goal_yield: new_goal_yield,
    };

    match diesel::insert_into(watchlists::table)
        .values(watchlist)
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

pub fn delete(watchlist_id: i32) {
    let mut connection = establish_connection();

    let deletion = diesel::delete(watchlists.filter(id.eq(watchlist_id))).execute(&mut connection);

    match deletion {
        Ok(a) => info!("Deleted row in watchlists with ID: {:?}", a),
        Err(e) => error!("Error while deleting row in watchlists: {:?}", e),
    }
}

pub async fn update_yield(target_id: i32, new_yield: f64) -> Result<(), anyhow::Error> {
    let connection = &mut establish_connection();

    diesel::update(watchlists)
        .filter(id.eq(target_id))
        .set(goal_yield.eq(new_yield))
        .execute(connection)?;

    Ok(())
}

pub fn get_all() -> Vec<Watchlist> {
    let connection = &mut establish_connection();

    let all_watchlists: Result<Vec<Watchlist>, diesel::result::Error> = watchlists::table
        .select(watchlists::table::all_columns())
        .get_results(connection);

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
    let mut connection = &mut establish_connection();

    let r: Vec<Watchlist> = watchlists
        .filter(user_id.eq(id_))
        .select(Watchlist::as_select())
        .load(connection)
        .expect("Error loading watchlists for user}");

    r
}
