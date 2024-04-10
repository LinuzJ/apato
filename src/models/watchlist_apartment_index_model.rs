use crate::models::watchlist::Watchlist;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = crate::db::schema::watchlist_apartment_index)]
pub struct InsertableWatchlistApartmentIndex {
    pub watchlist_id: i32,
    pub card_id: i32,
    pub has_been_sent: bool,
}

#[derive(Debug, Clone, PartialEq, Associations, Identifiable, Queryable, Selectable)]
#[diesel(belongs_to(Watchlist, foreign_key = id))]
#[diesel(table_name = crate::db::schema::watchlist_apartment_index)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WatchlistApartmentIndex {
    pub id: i32,
    pub watchlist_id: i32,
    pub card_id: i32,
    pub has_been_sent: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
