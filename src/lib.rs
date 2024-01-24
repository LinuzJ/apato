#[macro_use]
extern crate rocket;
extern crate chrono;
extern crate diesel;
extern crate rocket_sync_db_pools;

mod db;
mod interest_rate;
mod logger;
mod models;
mod oikotie;
mod producer;
mod routes;

use crate::logger::setup_logger;
use producer::pricing_producer::PricingProducer;
use rocket::{tokio, Build, Rocket};

#[launch]
pub async fn rocket() -> Rocket<Build> {
    // Initialize logger
    let _ = setup_logger();

    tokio::spawn(async { PricingProducer::run().await });

    // Initialize Rocket app
    rocket::build().mount("/api", routes![routes::index::index])
}
