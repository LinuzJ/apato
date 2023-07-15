use crate::modules::apartment::Apartment;
use diesel::{PgConnection, RunQueryDsl};

use super::schema::apartments;

pub fn insert(conn: &mut PgConnection, apartment: Apartment) {
    diesel::insert_into(apartments::table)
        .values(&apartment)
        .execute(conn)
        .expect("Error creating article");
}
