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
use tokio::sync::watch;

pub fn insert(apartment: InsertableApartment) {
    let mut con = establish_connection();

    match diesel::insert_into(apartments::table)
        .values(apartment)
        .execute(&mut con)
    {
        Ok(n) => println!("Inserted {:?} rows into apartmens table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}

pub fn get_all_for_watchlist(watchlist: i32) -> Vec<Apartment> {
    let mut con = establish_connection();

    let all_apartments: Result<Vec<Apartment>, Error> = apartments::table
        .filter(watchlist_id.eq(watchlist))
        .select(apartments::table::all_columns())
        .load(&mut con);
    match all_apartments {
        Ok(n) => {
            println!(
                "Fetched {:?} apartments for watchlist {:?}",
                n.len(),
                watchlist
            );
            n
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Vec::new()
        }
    }
}

pub fn get_all_valid_for_watchlist(watchlist: i32) -> Vec<Apartment> {
    let con = &mut establish_connection();

    let watchlist_from_db = watchlists::table
        .filter(watchlists::id.eq(watchlist))
        .select(Watchlist::as_select())
        .get_result(con);
    println!("ABCABC {:?}", watchlist_from_db);
    // let valid_apartments: Result<Vec<Apartment>, Error> =
    //     Apartment::belonging_to(&watchlist_from_db)
    //         .select(Apartment::as_select())
    //         .load(con)?;

    // match valid_apartments {
    //     Ok(n) => {
    //         println!(
    //             "Fetched {:?} apartments for watchlist {:?}",
    //             n.len(),
    //             watchlist
    //         );
    //         n
    //     }
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //         Vec::new()
    //     }
    // }
    let res: Vec<Apartment> = Vec::new();
    res
}
