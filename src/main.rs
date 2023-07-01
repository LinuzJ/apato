#[macro_use]
extern crate rocket;

mod helpers;
mod marketplace_client;
mod oikotie_client;
mod tokens;
use marketplace_client::{Apartment, Location};
use oikotie_client::OikotieClient;

#[get("/")]
async fn index() -> String {
    let oikotie_client: OikotieClient = OikotieClient { tokens: None };
    let location: Location = Location {
        id: 1651,
        level: 4,
        name: String::from("Etu-Töölö"),
    };
    let apartments: Vec<marketplace_client::Apartment> =
        oikotie_client.get_apartments(location).await;
    println!("{:?}", apartments);
    String::from("oogalaboogala")
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build().mount("/", routes![index]).launch().await?;

    Ok(())
}
