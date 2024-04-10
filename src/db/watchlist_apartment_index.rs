use std::sync::Arc;

use diesel::{prelude::*, result::Error};
use log::{error, info};

use super::{
    establish_connection, schema::watchlist_apartment_index,
    schema::watchlist_apartment_index::dsl::*,
};
use crate::{
    config::Config,
    models::{
        watchlist::Watchlist,
        watchlist_apartment_index_model::{
            self, InsertableWatchlistApartmentIndex, WatchlistApartmentIndex,
        },
    },
};

pub fn insert(config: &Arc<Config>, target_watchlist_id: i32, target_card_id: i32) {
    let mut conn = establish_connection(config);

    let insertable = InsertableWatchlistApartmentIndex {
        watchlist_id: target_watchlist_id,
        card_id: target_card_id,
        has_been_sent: false,
    };

    match diesel::insert_into(watchlist_apartment_index::table)
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

    let valid_connectors: Result<Vec<WatchlistApartmentIndex>, Error> =
        watchlist_apartment_index::table
            .filter(watchlist_apartment_index::card_id.eq(target_card_id))
            .filter(watchlist_apartment_index::watchlist_id.eq(watchlist.id))
            .select(WatchlistApartmentIndex::as_select())
            .load(conn);

    valid_connectors
}

pub fn get_unsent_apartments(
    config: &Arc<Config>,
    watchlist: &Watchlist,
) -> Result<Vec<i32>, Error> {
    let conn = &mut establish_connection(config);

    let valid_connectors: Result<Vec<i32>, Error> = watchlist_apartment_index::table
        .filter(watchlist_apartment_index::has_been_sent.eq(false))
        .filter(watchlist_apartment_index::watchlist_id.eq(watchlist.id))
        .select(card_id)
        .load::<i32>(conn);

    valid_connectors
}
