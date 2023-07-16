use crate::{
    clients::oikotie_client::{Location, OikotieClient},
    db::{self, Db},
    modules::apartment::Apartment,
};
use std::time::Duration;

use rocket::{tokio::time, Rocket};

pub struct Producer {}

impl Producer {
    pub async fn run(rocket: &Rocket<rocket::Orbit>) {
        let mut interval = time::interval(Duration::from_secs(60));

        // db::establish_connection();

        loop {
            // let mut db: Db = Db::get_one(rocket).await.unwrap();
            let oikotie_client: OikotieClient = OikotieClient::new().await;

            let location: Location = Location {
                id: 1645,
                level: 4,
                name: String::from("Ullanlinna"),
            };

            let apartments: Vec<Apartment> = oikotie_client.get_apartments(location, false).await;

            // for ele in apartments {
            //     db::apartment::insert(&mut *db, ele);
            // }
            interval.tick().await;
        }
    }
}
