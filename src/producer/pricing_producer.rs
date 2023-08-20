use crate::{
    db::{self, establish_connection, watchlist},
    models::apartment::Apartment,
    oikotie::oikotie_client::{Location, OikotieClient},
};
use log::info;
use std::time::{Duration, Instant};

use rocket::tokio::time;

pub struct PricingProducer {}

impl PricingProducer {
    pub async fn run() -> ! {
        let mut interval = time::interval(Duration::from_secs(120));

        // TEMP Initialization
        let watchlists = watchlist::get_all();
        if watchlists.len() == 0 {
            let new_location: Location = Location {
                id: 1645,
                level: 4,
                name: String::from("Ullanlinna"),
            };

            watchlist::create(new_location);
        }

        loop {
            /*  TODO
             *    - get watchlists -> locations
             *    - for each location -> calculate prices
             *    - insert into apartments table
             *    - /api/{watchlist} -> summary of apartments
             *    - /api/add_watchlist -> adds watchlist
             */
            info!("Starting PricingProducer run");
            let start = Instant::now();

            let watchlists = watchlist::get_all();

            for watchlist in watchlists {
                info!(
                    "Starting calculating prices for watchlist_id: {:?}",
                    watchlist.id
                );

                let oikotie_client: OikotieClient = OikotieClient::new().await;

                let apartments = oikotie_client
                    .get_apartments(watchlist.clone(), false)
                    .await;

                handle_apartments(apartments);

                info!(
                    "Finished price calculations for watchlist_id: {:?}",
                    watchlist.id
                );
            }

            let duration = start.elapsed();
            info!("Finished PricingProducer run in {:?}", duration);

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
