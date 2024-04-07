use std::sync::Arc;

use super::{
    establish_connection, schema::apartments, schema::apartments::dsl::*, schema::watchlists,
    watchlist::check_user,
};
use crate::{
    config::Config,
    models::{apartment::Apartment, apartment::InsertableApartment, watchlist::Watchlist},
};
use anyhow::anyhow;
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
    user: i32,
    watchlist: i32,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let mut con = establish_connection(config);
    let correct_user = check_user(config, user, watchlist);
    if !correct_user {
        return Err(anyhow!("Error: Wrong user"));
    }

    let all_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(watchlist_id.eq(watchlist))
        .select(apartments::table::all_columns())
        .load(&mut con);

    return Ok(all_apartments?);
}

pub fn get_all_valid_for_watchlist(
    config: &Arc<Config>,
    user: i32,
    watchlist: i32,
) -> Result<Vec<Apartment>, anyhow::Error> {
    let con = &mut establish_connection(config);
    let correct_user = check_user(config, user, watchlist);
    if !correct_user {
        return Err(anyhow!("Error: Wrong user"));
    }

    let watchlist_from_db = watchlists::table
        .filter(watchlists::id.eq(watchlist))
        .select(Watchlist::as_select())
        .first(con)
        .optional();
    // TODO -> Make this one join query. diesel fucked the join fsr
    let target_watchlist: Watchlist = match watchlist_from_db {
        Ok(Some(w)) => w,
        Ok(None) => return Err(anyhow!("Error: Did not find a watchlist")),
        Err(_) => return Err(anyhow!("Error: Did not find a watchlist")),
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

    return Ok(valid_apartments?);
}
