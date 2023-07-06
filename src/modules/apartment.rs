use crate::db::schema::apartments;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable, Insertable)]
#[table_name = "apartments"]
pub struct Apartment {
    pub id: String,
    pub location_id: i32,
    pub location_level: i32,
    pub location_name: String,
    pub size: f32,
    pub rooms: i32,
    pub price: String,
    pub additional_costs: i32,
}
