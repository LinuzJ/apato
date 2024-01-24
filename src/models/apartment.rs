use crate::db::schema::apartments;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable, Insertable, Clone)]
#[diesel(table_name = apartments)]
pub struct Apartment {
    pub card_id: String,
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub size: f64,
    pub rooms: i32,
    pub price: i32,
    pub additional_costs: i32,
    pub rent: i32,
    pub watchlist_id: i32,
}
