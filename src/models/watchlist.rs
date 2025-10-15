use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Insertable)]
#[diesel(table_name = crate::db::schema::watchlists)]
pub struct InsertableWatchlist {
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub chat_id: i64,
    pub target_yield: Option<f64>,
    pub target_size_min: Option<i32>,
    pub target_size_max: Option<i32>,
}

#[derive(Debug, Queryable, Selectable, Identifiable, Clone, Serialize)]
#[diesel(table_name = crate::db::schema::watchlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Watchlist {
    pub id: i32,
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub chat_id: i64,
    pub target_yield: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub target_size_min: Option<i32>,
    pub target_size_max: Option<i32>,
}

pub struct SizeTarget {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl SizeTarget {
    pub fn empty() -> Self {
        SizeTarget {
            min: None,
            max: None,
        }
    }
}
