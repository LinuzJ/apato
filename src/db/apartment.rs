use std::sync::Arc;

use super::{
    establish_connection,
    schema::apartments,
    schema::apartments::dsl::*,
    watchlist::{check_chat, get_watchlist},
};
use crate::{
    config::Config,
    models::{apartment::Apartment, apartment::InsertableApartment, watchlist::Watchlist},
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};

pub fn insert(config: &Arc<Config>, apartment: InsertableApartment) {
    let mut con = establish_connection(config);

    match diesel::insert_into(apartments::table)
        .values(apartment)
        .execute(&mut con)
    {
        Ok(n) => println!("Inserted {:?} rows into apartments table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}

pub fn get_all_for_watchlist(
    config: &Arc<Config>,
    chat_id: i64,
    watchlist: i32,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let mut con = establish_connection(config);
    let correct_chat = check_chat(config, chat_id, watchlist);
    if !correct_chat {
        return Err(anyhow!("Error: Wrong chat"));
    }

    let all_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(watchlist_id.eq(watchlist))
        .select(apartments::table::all_columns())
        .load(&mut con);

    Ok(all_apartments?)
}

pub fn get_all_valid_for_watchlist(
    config: &Arc<Config>,
    chat_id: i64,
    watchlist: i32,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let con = &mut establish_connection(config);
    let correct_chat = check_chat(config, chat_id, watchlist);

    if !correct_chat {
        return Err(anyhow!("Error: Wrong chat"));
    }

    let potential_watchlist = get_watchlist(config, watchlist);

    let target_watchlist = match potential_watchlist {
        Ok(w) => w,
        Err(_e) => return Err(anyhow!("No watchlist wound with this name")),
    };

    // let valid_apartments: Result<Vec<Apartment>, Error> =
    //     Apartment::belonging_to(&watchlist_from_db)
    //         .select(Apartment::as_select())
    //         .load(con)?;
    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::watchlist_id.eq(target_watchlist.id))
        .filter(apartments::estimated_yield.gt(target_watchlist.goal_yield.unwrap()))
        .select(Apartment::as_select())
        .load(con);

    Ok(valid_apartments?)
}

pub fn get_new_for_watchlist(
    config: &Arc<Config>,
    watchlist: Watchlist,
    interval_start_time: NaiveDateTime,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let conn = &mut establish_connection(config);

    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::watchlist_id.eq(watchlist.id))
        .filter(apartments::estimated_yield.gt(watchlist.goal_yield.unwrap()))
        .filter(apartments::created_at.gt(interval_start_time))
        .select(Apartment::as_select())
        .load(conn);

    Ok(valid_apartments?)
}

pub fn get_apartments_within_period(
    config: &Arc<Config>,
    wanted_card_id: String,
    interval_start_time: NaiveDateTime,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let conn = &mut establish_connection(config);

    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::card_id.eq(Some(wanted_card_id)))
        .filter(apartments::created_at.gt(interval_start_time))
        .select(Apartment::as_select())
        .load(conn);

    Ok(valid_apartments?)
}
