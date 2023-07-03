use crate::modules::marketplace_client::{self, Location};
use crate::modules::oikotie_client::OikotieClient;
use std::time::Duration;

use rocket::tokio::{self, time};

pub async fn run() {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        println!("Here I am");

        loop {
            println!("New loop");
            let oikotie_client: OikotieClient = OikotieClient::new().await;

            let location: Location = Location {
                id: 1645,
                level: 4,
                name: String::from("Ullanlinna"),
            };

            let apartments: Vec<marketplace_client::Apartment> =
                oikotie_client.get_apartments(location, false).await;

            println!("{:?}", apartments);
            interval.tick().await;
        }
    });
}
