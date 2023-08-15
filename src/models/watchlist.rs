use chrono::NaiveDateTime;
use diesel::Queryable;

#[derive(Debug, Queryable)]
pub struct Watchlist {
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
