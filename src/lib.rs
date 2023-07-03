#[macro_use]
extern crate rocket;
use modules::producer;
use rocket::serde::json::{json, Value};
use rocket::{Build, Rocket};

mod modules;
mod routes;

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[launch]
pub async fn rocket() -> Rocket<Build> {
    let _ = producer::run().await;

    rocket::build()
        .mount("/", routes![routes::index::index])
        .register(".", catchers![not_found])
}
