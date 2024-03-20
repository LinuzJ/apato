#[macro_use]
extern crate tokio;
extern crate chrono;
extern crate diesel;
extern crate rocket_sync_db_pools;

pub mod bot;
mod db;
mod interest_rate;
pub mod logger;
pub mod models;
mod oikotie;
pub mod producer;

pub use producer::calculate_rental_yield;
use producer::pricing_producer::PricingProducer;

pub async fn spawn_apato() -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async { PricingProducer::run().await })
}
