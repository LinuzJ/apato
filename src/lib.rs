#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate rocket_sync_db_pools;

mod db;
mod modules;
mod pricing;
mod routes;

use pricing::pricing_producer::PricingProducer;
use rocket::{fairing::AdHoc, Build, Rocket};

#[launch]
pub async fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(AdHoc::on_liftoff(
            "Pricing producer",
            |rocket: &Rocket<rocket::Orbit>| {
                Box::pin(async move {
                    PricingProducer::run(rocket).await;
                })
            },
        ))
        .mount("/api", routes![routes::index::index])
    // .attach(db::Db::fairing())
}
