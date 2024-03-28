use super::{
    establish_connection,
    schema::apartments,
    schema::watchlists,
    schema::{self, apartments::dsl::*, watchlists::dsl::*},
};
use crate::models::{
    apartment::Apartment,
    apartment::InsertableApartment,
    watchlist::{self, Watchlist},
};
use diesel::{prelude::*, result::Error};
use log::error;
use tokio::sync::watch;

pub fn insert(apartment: InsertableApartment) {
    let mut con = establish_connection();

    match diesel::insert_into(apartments::table)
        .values(apartment)
        .execute(&mut con)
    {
        Ok(n) => println!("Inserted {:?} rows into apartments table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}

pub fn get_all_for_watchlist(watchlist: i32) -> Result<Vec<Apartment>, Error> {
    let mut con = establish_connection();

    let all_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(watchlist_id.eq(watchlist))
        .select(apartments::table::all_columns())
        .load(&mut con);

    return all_apartments;
}

pub fn get_all_valid_for_watchlist(watchlist: i32) -> Result<Vec<Apartment>, Error> {
    let con = &mut establish_connection();

    let watchlist_from_db = watchlists::table
        .filter(watchlists::id.eq(watchlist))
        .select(Watchlist::as_select())
        .first(con)
        .optional();

    let target_watchlist_id = match watchlist_from_db {
        Ok(Some(w)) => w.id,
        Ok(None) => 0,
        Err(_) => {
            error!("Error while trying to fetch watchlist");
            0
        }
    };

    // let valid_apartments: Result<Vec<Apartment>, Error> =
    //     Apartment::belonging_to(&watchlist_from_db)
    //         .select(Apartment::as_select())
    //         .load(con)?;
    let valid_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(apartments::watchlist_id.eq(target_watchlist_id))
        .select(Apartment::as_select())
        .load(con);

    return valid_apartments;
}
