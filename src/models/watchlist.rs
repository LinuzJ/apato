use chrono::NaiveDateTime;
use diesel::Queryable;

#[derive(Debug, Queryable, Clone)]
#[diesel(table_name = watchlists)]
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
