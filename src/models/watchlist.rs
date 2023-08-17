use chrono::{DateTime, Utc};
use diesel::Queryable;

#[derive(Debug, Queryable)]
pub struct Watchlist {
    pub id: i32,
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
