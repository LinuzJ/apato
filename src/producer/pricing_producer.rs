use crate::{
    db::watchlist,
    oikotie::oikotie::{Location, Oikotie},
    producer::calculations::calculate_yields_for_apartments,
};
use log::info;
use std::time::{Duration, Instant};

use rocket::tokio::time;

pub struct PricingProducer {}

impl PricingProducer {
    pub async fn run() -> ! {
        let interval_in_seconds = 5 * 60;
        let mut interval = time::interval(Duration::from_secs(interval_in_seconds));

        // TEMP Initialization
        let watchlists = watchlist::get_all();
        if watchlists.len() == 0 {
            watchlist::create(Location {
                id: 1645,
                level: 4,
                name: String::from("Ullanlinna"),
            });
        }

        loop {
            /*  TODO
             *    - get watchlists -> locations - DONE
             *    - for each location -> calculate price
             *    - Smarter producer/consumer logic for handling watchlist calculations?
             *    - insert into apartments table - DONE
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

                let mut oikotie_client: Oikotie = Oikotie::new().await;

                let apartments = oikotie_client.get_apartments(&watchlist).await;

                calculate_yields_for_apartments(apartments, oikotie_client).await;

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
