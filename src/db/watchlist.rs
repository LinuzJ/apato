use crate::{models::watchlist::Watchlist, pricing::oikotie_client::Location};
use diesel::{PgConnection, RunQueryDsl};

use super::schema::watchlists;

pub fn insert(conn: &mut PgConnection, location: Location) {
    let insertable: &Watchlist = &Watchlist {
        location_id: location.id,
        location_level: location.level,
        location_name: location.name,
        created_at: todo!(),
        updated_at: todo!(),
    };
    match diesel::insert_into(watchlists::table)
        .values(location)
        .execute(conn)
    {
        Ok(n) => println!("Inserted {:?} rows into watchlist table", n),
        Err(e) => println!("Error: {:?}", e),
    }
}
