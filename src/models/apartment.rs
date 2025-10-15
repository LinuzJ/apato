use crate::models::watchlist::Watchlist;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::db::schema::apartments)]
pub struct InsertableApartment {
    pub card_id: i32,
    pub location_id: Option<i32>,
    pub location_level: Option<i32>,
    pub location_name: Option<String>,
    pub size: Option<f64>,
    pub rooms: Option<i32>,
    pub price: Option<i32>,
    pub additional_costs: Option<i32>,
    pub rent: Option<i32>,
    pub estimated_yield: Option<f64>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Associations, Identifiable, Queryable, Selectable, Serialize)]
#[diesel(belongs_to(Watchlist, foreign_key = id))]
#[diesel(table_name = crate::db::schema::apartments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Apartment {
    pub id: i32,
    pub card_id: i32,
    pub location_id: Option<i32>,
    pub location_level: Option<i32>,
    pub location_name: Option<String>,
    pub size: Option<f64>,
    pub rooms: Option<i32>,
    pub price: Option<i32>,
    pub additional_costs: Option<i32>,
    pub rent: Option<i32>,
    pub estimated_yield: Option<f64>,
    pub url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
