use std::sync::Arc;

use super::{establish_connection, schema::watchlists, schema::watchlists::dsl::*};
use crate::config::Config;
use crate::models::watchlist::{InsertableWatchlist, SizeTarget};
use crate::{models::watchlist::Watchlist, oikotie::oikotie::Location};
use anyhow::anyhow;
use diesel::prelude::*;

use log::error;
use log::info;

pub fn insert(
    config: &Arc<Config>,
    location: Location,
    new_chat_id: i64,
    new_target_yield: Option<f64>,
    target_size: Option<SizeTarget>,
) {
    let mut connection = establish_connection(config);

    let min_size = None;
    let max_size = None;

    match target_size {
        Some(s) => {
            min_size = Some(s.min);
            max_size = Some(s.max);
        }
        None => {}
    }

    let watchlist: InsertableWatchlist = InsertableWatchlist {
        location_id: location.id,
        location_level: location.level,
        location_name: location.name,
        chat_id: new_chat_id,
        target_yield: new_target_yield,
        target_size_min: min_size,
        target_size_max: max_size,
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

pub fn get_watchlist(config: &Arc<Config>, watchlist_id: i32) -> Result<Watchlist, anyhow::Error> {
    let conn = &mut establish_connection(config);

    let watchlist_from_db = watchlists::table
        .filter(watchlists::id.eq(watchlist_id))
        .select(Watchlist::as_select())
        .first(conn)
        .optional();

    match watchlist_from_db {
        Ok(Some(w)) => Ok(w),
        Ok(None) => Err(anyhow!("Error: Did not find a watchlist")),
        Err(_) => Err(anyhow!("Error: Did not find a watchlist")),
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
        .set(target_yield.eq(new_yield))
        .execute(connection)?;

    Ok(())
}

pub fn get_all(config: &Arc<Config>) -> Vec<Watchlist> {
    let connection = &mut establish_connection(config);

    let all_watchlists: Result<Vec<Watchlist>, diesel::result::Error> = watchlists::table
        .select(watchlists::table::all_columns())
        .get_results(connection);

    match all_watchlists {
        Ok(w) => w,
        Err(e) => {
            println!("Error: {:?}", e);
            Vec::new()
        }
    }
}

pub fn get_for_chat(config: &Arc<Config>, id_: i64) -> Vec<Watchlist> {
    let connection = &mut establish_connection(config);

    let r: Vec<Watchlist> = watchlists
        .filter(chat_id.eq(id_))
        .select(Watchlist::as_select())
        .load(connection)
        .expect("Error loading watchlists for chat}");

    r
}

pub fn get_for_chat_and_location(
    config: &Arc<Config>,
    id_: i64,
    location: &String,
) -> Vec<Watchlist> {
    let connection = &mut establish_connection(config);

    let r: Vec<Watchlist> = watchlists
        .filter(chat_id.eq(id_))
        .filter(location_name.eq(location))
        .select(Watchlist::as_select())
        .load(connection)
        .expect("Error loading watchlists for chat}");

    r
}

pub fn check_chat(config: &Arc<Config>, chat_id_to_check: i64, watchlist: i32) -> bool {
    let con = &mut establish_connection(config);

    let watchlist_from_db: Vec<Watchlist> = watchlists
        .filter(id.eq(watchlist))
        .filter(chat_id.eq(chat_id_to_check))
        .select(Watchlist::as_select())
        .load(con)
        .expect("Error loading watchlists for chat}");

    !watchlist_from_db.is_empty()
}
