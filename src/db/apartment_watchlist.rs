use std::sync::Arc;

use diesel::{prelude::*, result::Error};
use log::{error, info};

use super::{
    establish_connection, schema::apartment_watchlist, schema::apartment_watchlist::dsl::*,
};
use crate::{
    config::Config,
    models::{
        apartment_watchlist_model::{InsertableWatchlistApartmentIndex, WatchlistApartmentIndex},
        watchlist::Watchlist,
    },
};

pub fn insert(config: &Arc<Config>, target_watchlist_id: i32, target_card_id: i32) {
    let mut conn = establish_connection(config);

    let insertable = InsertableWatchlistApartmentIndex {
        watchlist_id: target_watchlist_id,
        card_id: target_card_id,
        has_been_sent: false,
    };

    match diesel::insert_into(apartment_watchlist::table)
        .values(insertable)
        .execute(&mut conn)
    {
        Ok(n) => info!("Inserted {:?} rows into apartments table", n),
        Err(e) => error!("Error: {:?}", e),
    }
}

pub fn get_watchlist_apartment_connector(
    config: &Arc<Config>,
    watchlist: &Watchlist,
    target_card_id: i32,
) -> Result<Vec<WatchlistApartmentIndex>, Error> {
    let conn = &mut establish_connection(config);

    let valid_connectors: Result<Vec<WatchlistApartmentIndex>, Error> = apartment_watchlist::table
        .filter(apartment_watchlist::card_id.eq(target_card_id))
        .filter(apartment_watchlist::watchlist_id.eq(watchlist.id))
        .select(WatchlistApartmentIndex::as_select())
        .load(conn);

    valid_connectors
}

pub fn get_unsent_apartments(
    config: &Arc<Config>,
    watchlist: &Watchlist,
) -> Result<Vec<i32>, Error> {
    let conn = &mut establish_connection(config);

    let valid_connectors: Result<Vec<i32>, Error> = apartment_watchlist::table
        .filter(apartment_watchlist::has_been_sent.eq(false))
        .filter(apartment_watchlist::watchlist_id.eq(watchlist.id))
        .select(card_id)
        .load::<i32>(conn);

    valid_connectors
}

pub fn set_to_read(config: &Arc<Config>, watchlist: &Watchlist, target_card_id: i32) {
    let conn = &mut establish_connection(config);

    match diesel::update(
        apartment_watchlist
            .filter(watchlist_id.eq(watchlist.id))
            .filter(card_id.eq(target_card_id)),
    )
    .set(has_been_sent.eq(true))
    .execute(conn)
    {
        Ok(_n) => info!(
            "Consumer set watchlist {:?} and card_id {} to has_been_sent = {}",
            watchlist.id, target_card_id, true
        ),
        Err(e) => error!("Error: {:?}", e),
    }
}

pub fn exists(
    config: &Arc<Config>,
    target_watchlist_id: i32,
    target_card_id: i32,
) -> Result<bool, Error> {
    let conn = &mut establish_connection(config);

    let valid_apartments: Result<Vec<WatchlistApartmentIndex>, Error> = apartment_watchlist::table
        .filter(apartment_watchlist::card_id.eq(target_card_id))
        .filter(apartment_watchlist::watchlist_id.eq(target_watchlist_id))
        .select(WatchlistApartmentIndex::as_select())
        .limit(1)
        .load(conn);

    match valid_apartments {
        Ok(aps) => Ok(aps.len() == 1),
        Err(e) => Err(e),
    }
}
