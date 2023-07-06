#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate rocket_sync_db_pools;

mod clients;
mod db;
mod modules;
mod routes;

use clients::producer::Producer;
use rocket::{Build, Rocket};

#[launch]
pub async fn rocket() -> Rocket<Build> {
    Producer::run().await;

    rocket::build()
        .attach(db::Db::fairing())
        .mount("/api", routes![routes::index::index])
}
