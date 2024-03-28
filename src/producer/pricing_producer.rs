use crate::{
    db::watchlist,
    models::apartment::InsertableApartment,
    oikotie::oikotie::{Location, Oikotie},
    producer::calculations::process_apartment_calculations,
};
use log::info;
use std::time::{Duration, Instant};
use tokio::time;

pub struct PricingProducer {}

impl PricingProducer {
    pub async fn run() {
        let interval_in_seconds = 5 * 60;
        let mut interval = time::interval(Duration::from_secs(interval_in_seconds));

        let watchlists = watchlist::get_all();
        // TEMP Initialization of a watchlist
        if watchlists.len() == 0 {
            watchlist::insert(
                Location {
                    id: 1645,
                    level: 4,
                    name: String::from("Ullanlinna"),
                },
                1,
                Some(2.0),
            );
        }

        loop {
            info!("Starting PricingProducer run");
            let start = Instant::now();

            let watchlists = watchlist::get_all();

            for watchlist in watchlists {
                info!(
                    "Starting calculating prices for watchlist_id: {:?}",
                    watchlist.id
                );

                let mut oikotie_client: Oikotie = Oikotie::new().await;

                let apartments: Option<Vec<InsertableApartment>> =
                    oikotie_client.get_apartments(&watchlist).await;

                process_apartment_calculations(apartments, oikotie_client).await;

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
