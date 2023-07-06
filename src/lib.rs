#[macro_use]
extern crate rocket;
extern crate diesel;

mod clients;
mod routes;

use clients::producer::Producer;
use rocket::{Build, Rocket};

#[launch]
pub async fn rocket() -> Rocket<Build> {
    Producer::run().await;

    rocket::build().mount("/api", routes![routes::index::index])
}
