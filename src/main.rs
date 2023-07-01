#[macro_use]
extern crate rocket;

mod helpers;
mod marketplace_client;
mod oikotie_client;
mod tokens;
use oikotie_client::OikotieClient;

#[get("/")]
async fn index() -> String {
    let _oikotie_client: OikotieClient = OikotieClient { tokens: None };
    String::from("abc")
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build().mount("/", routes![index]).launch().await?;

    Ok(())
}
