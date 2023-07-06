#[macro_use]
extern crate rocket;

mod modules;
mod routes;

use modules::producer::Producer;
use rocket::{Build, Rocket};

#[launch]
pub async fn rocket() -> Rocket<Build> {
    Producer::run().await;

    rocket::build().mount("/api", routes![routes::index::index])
}
