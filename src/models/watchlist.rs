use crate::db::schema::watchlists;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable, Insertable)]
#[table_name = "watchlists"]
pub struct Watchlist {
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
