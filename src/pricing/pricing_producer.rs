use crate::{
    db::{self, establish_connection, schema::apartments},
    modules::apartment::Apartment,
    pricing::oikotie_client::{Location, OikotieClient},
};
use std::time::Duration;

use diesel::PgConnection;
use rocket::{tokio::time, Rocket};

pub struct PricingProducer {}

impl PricingProducer {
    pub async fn run() {
        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            let oikotie_client: OikotieClient = OikotieClient::new().await;

            let location: Location = Location {
                id: 1645,
                level: 4,
                name: String::from("Ullanlinna"),
            };

            let apartments = oikotie_client.get_apartments(location, false).await;

            handle_apartments(apartments);

            interval.tick().await;
        }
    }
}

fn handle_apartments(potential_apartments: Option<Vec<Apartment>>) {
    match potential_apartments {
        Some(apartments) => {
            for ele in apartments {
                db::apartment::insert(&mut establish_connection(), ele);
            }
        }
        None => println!("No apartments added.."),
    }
}
