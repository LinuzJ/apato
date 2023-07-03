use crate::modules::marketplace_client::{self, Location};
use crate::modules::oikotie_client::OikotieClient;
use std::time::Duration;

use rocket::tokio::{self, time};

pub struct Producer {}

impl Producer {
    pub async fn run() {
        tokio::spawn(async {
            let mut interval = time::interval(Duration::from_secs(60));

            loop {
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
}
