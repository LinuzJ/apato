use crate::modules::apartment::Apartment;
use diesel::{PgConnection, RunQueryDsl};

use super::schema::apartments;

pub fn insert(conn: &mut PgConnection, apartment: Apartment) {
    match diesel::insert_into(apartments::table)
        .values(&apartment)
        .execute(conn)
    {
        Ok(n) => println!("Inserted {:?} rows into apartmens table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}
