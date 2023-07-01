#[macro_use]
extern crate rocket;

mod helpers;
mod marketplace_client;
mod oikotie_client;
mod tokens;
use marketplace_client::{Location, MarketplaceClient};
use oikotie_client::OikotieClient;

#[get("/")]
async fn index() -> String {
    let oikotie_client: OikotieClient = OikotieClient { tokens: None };
    let location: Location = Location {
        id: 1651,
        level: 4,
        name: String::from("Etu-Töölö"),
    };
    oikotie_client.get_apartments(location).await;
    String::from("abc")
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build().mount("/", routes![index]).launch().await?;

    Ok(())
}
