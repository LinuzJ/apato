#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate rocket_sync_db_pools;

mod clients;
mod db;
mod modules;
mod routes;

use clients::producer::Producer;
use rocket::{fairing::AdHoc, Build, Rocket};

#[launch]
pub async fn rocket() -> Rocket<Build> {
    rocket::build()
        // .attach(db::Db::fairing())
        .attach(AdHoc::on_liftoff(
            "Background process",
            |rocket: &Rocket<rocket::Orbit>| {
                Box::pin(async move {
                    Producer::run(rocket).await;
                })
            },
        ))
        .mount("/api", routes![routes::index::index])
}
