use super::{establish_connection, schema::apartments, schema::apartments::dsl::*};
use crate::models::apartment::{Apartment, InsertableApartment};
use diesel::{prelude::*, result::Error};

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
