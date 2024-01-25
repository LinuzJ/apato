#[macro_use]
extern crate tokio;
extern crate chrono;
extern crate diesel;
extern crate rocket_sync_db_pools;

mod db;
mod interest_rate;
mod logger;
mod models;
mod oikotie;
pub mod producer;

use crate::logger::setup_logger;
pub use producer::calculate_rental_yield;
use producer::pricing_producer::PricingProducer;

pub async fn launch_apato() {
    // Initialize logger
    let _ = setup_logger();

    tokio::spawn(async { PricingProducer::run().await })
        .await
        .unwrap();
}
