use std::sync::Arc;

use super::{establish_connection, schema::watchlists, schema::watchlists::dsl::*};
use crate::config::Config;
use crate::models::watchlist::InsertableWatchlist;
use crate::{models::watchlist::Watchlist, oikotie::oikotie::Location};
use diesel::prelude::*;
use diesel::result::Error;
use log::error;
use log::info;

pub fn insert(
    config: &Arc<Config>,
    location: Location,
    new_user_id: i32,
    new_goal_yield: Option<f64>,
) {
    let mut connection = establish_connection(config);

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

pub fn delete(config: &Arc<Config>, watchlist_id: i32) {
    let mut connection = establish_connection(config);

    let deletion = diesel::delete(watchlists.filter(id.eq(watchlist_id))).execute(&mut connection);

    match deletion {
        Ok(a) => info!("Deleted row in watchlists with ID: {:?}", a),
        Err(e) => error!("Error while deleting row in watchlists: {:?}", e),
    }
}

pub async fn update_yield(
    config: &Arc<Config>,
    target_id: i32,
    new_yield: f64,
) -> Result<(), anyhow::Error> {
    let connection = &mut establish_connection(config);

    diesel::update(watchlists)
        .filter(id.eq(target_id))
        .set(goal_yield.eq(new_yield))
        .execute(connection)?;

    Ok(())
}

pub fn get_all(config: &Arc<Config>) -> Vec<Watchlist> {
    let connection = &mut establish_connection(config);

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

pub fn get_for_user(config: &Arc<Config>, id_: i32) -> Vec<Watchlist> {
    let connection = &mut establish_connection(config);

    let r: Vec<Watchlist> = watchlists
        .filter(user_id.eq(id_))
        .select(Watchlist::as_select())
        .load(connection)
        .expect("Error loading watchlists for user}");

    r
}

pub fn check_user(config: &Arc<Config>, user: i32, watchlist: i32) -> bool {
    let mut con = &mut establish_connection(config);

    let watchlist_from_db: Vec<Watchlist> = watchlists
        .filter(id.eq(watchlist))
        .select(Watchlist::as_select())
        .load(con)
        .expect("Error loading watchlists for user}");

    return watchlist_from_db[0].user_id == user;
}
