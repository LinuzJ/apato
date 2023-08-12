use crate::{
    db::{self, establish_connection, Db},
    modules::apartment::Apartment,
    pricing::oikotie_client::{Location, OikotieClient},
};
use std::time::Duration;

use diesel::PgConnection;
use rocket::{tokio::time, Rocket};

pub struct PricingProducer {}

impl PricingProducer {
    pub async fn run(rocket: &Rocket<rocket::Orbit>) {
        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            let oikotie_client: OikotieClient = OikotieClient::new().await;

            let location: Location = Location {
                id: 1645,
                level: 4,
                name: String::from("Ullanlinna"),
            };

            let apartments: Vec<Apartment> = oikotie_client.get_apartments(location, false).await;

            for ele in apartments {
                print!("Inserting");
                db::apartment::insert(&mut establish_connection(), ele);
            }
            interval.tick().await;
        }
    }
}
