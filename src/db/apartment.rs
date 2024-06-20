use std::sync::Arc;

use super::{
    establish_connection,
    schema::{
        apartments::{self, dsl::*},
        watchlist_apartment_index,
    },
    watchlist::{check_chat, get_watchlist},
};
use crate::{
    config::Config,
    models::{apartment::Apartment, apartment::InsertableApartment},
};
use anyhow::anyhow;
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{prelude::*, result::Error};
use log::{error, info};

pub fn insert(config: &Arc<Config>, apartment: InsertableApartment) {
    let mut conn = establish_connection(config);

    match diesel::insert_into(apartments::table)
        .values(apartment)
        .execute(&mut conn)
    {
        Ok(n) => info!("Inserted {:?} rows into apartments table", n),
        Err(e) => error!("Error: {:?}", e),
    }
}

pub fn get_all_for_watchlist(
    config: &Arc<Config>,
    chat_id: i64,
    watchlist_id_: i32,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let conn = &mut establish_connection(config);

    if !check_chat(config, chat_id, watchlist_id_) {
        return Err(anyhow!("Error: Wrong chat"));
    }

    let apartments_in_watchlist = watchlist_apartment_index::table
        .inner_join(
            apartments::table.on(watchlist_apartment_index::card_id.eq(apartments::card_id)),
        )
        .filter(watchlist_apartment_index::watchlist_id.eq(watchlist_id_))
        .select(Apartment::as_select())
        .load::<Apartment>(conn);

    Ok(apartments_in_watchlist?)
}

pub fn get_matching_for_watchlist(
    config: &Arc<Config>,
    chat_id: i64,
    watchlist_id_: i32,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let conn = &mut establish_connection(config);

    if !check_chat(config, chat_id, watchlist_id_) {
        return Err(anyhow!("Error: Wrong chat"));
    }

    let target_watchlist = match get_watchlist(config, watchlist_id_) {
        Ok(w) => w,
        Err(_e) => return Err(anyhow!("No watchlist wound with this name")),
    };

    let matching_apartments = watchlist_apartment_index::table
        .inner_join(
            apartments::table.on(watchlist_apartment_index::card_id.eq(apartments::card_id)),
        )
        .filter(watchlist_apartment_index::watchlist_id.eq(watchlist_id_))
        .filter(apartments::estimated_yield.gt(target_watchlist.target_yield.unwrap()))
        .select(Apartment::as_select())
        .load::<Apartment>(conn);

    Ok(matching_apartments?)
}

pub fn _get_apartments_within_period(
    config: &Arc<Config>,
    wanted_card_id: i32,
    interval_start_time: NaiveDateTime,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let conn = &mut establish_connection(config);

    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::card_id.eq(wanted_card_id))
        .filter(apartments::created_at.gt(interval_start_time))
        .select(Apartment::as_select())
        .load(conn);

    Ok(valid_apartments?)
}

pub fn get_apartment_by_card_id(
    config: &Arc<Config>,
    target_card_id: i32,
) -> Result<Option<Apartment>, Error> {
    let conn = &mut establish_connection(config);

    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::card_id.eq(target_card_id))
        .select(Apartment::as_select())
        .limit(1)
        .load(conn);

    match valid_apartments {
        Ok(aps) => {
            if !aps.is_empty() {
                let target_ap = aps[0].clone();
                Ok(Some(target_ap))
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn apartment_is_fresh(config: &Arc<Config>, target_card_id: i32) -> Result<bool, Error> {
    let conn = &mut establish_connection(config);
    let now = Utc::now().naive_local();
    let freshness_cutoff = now - Duration::days(5);

    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::card_id.eq(target_card_id))
        .filter(apartments::updated_at.gt(freshness_cutoff))
        .select(Apartment::as_select())
        .limit(1)
        .load(conn);

    match valid_apartments {
        Ok(aps) => Ok(aps.len() == 1),
        Err(e) => Err(e),
    }
}

pub fn update_yield(config: &Arc<Config>, target_card_id: i32, new_yield: f64) {
    let conn = &mut establish_connection(config);
    let update_res = diesel::update(apartments)
        .filter(apartments::card_id.eq(target_card_id))
        .set(apartments::estimated_yield.eq(Some(new_yield)))
        .execute(conn);

    match update_res {
        Ok(_n) => info!(
            "Consumer set apartment with card_id {:?} to yield = {}",
            target_card_id, new_yield
        ),
        Err(e) => error!("Error: {:?}", e),
    }
}
