use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::db::schema::watchlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Watchlist {
    pub id: i32,
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub user_id: i32,
    pub goal_yield: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
