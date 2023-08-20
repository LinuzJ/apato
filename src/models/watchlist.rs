use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::Queryable;

#[derive(Debug, Queryable, Clone)]
pub struct Watchlist {
    pub id: i32,
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
