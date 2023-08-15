use super::schema::apartments;
use crate::models::apartment::Apartment;
use diesel::{PgConnection, RunQueryDsl};

pub fn insert(conn: &mut PgConnection, apartment: Apartment) {
    match diesel::insert_into(apartments::table)
        .values(&apartment)
        .execute(conn)
    {
        Ok(n) => println!("Inserted {:?} rows into apartmens table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}
