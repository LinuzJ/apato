use crate::{
    config::Config,
    db::watchlist,
    models::apartment::InsertableApartment,
    oikotie::oikotie::{Location, Oikotie},
    producer::calculations::process_apartment_calculations,
};
use anyhow::Result;
use log::info;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time;

pub struct PricingProducer {}

impl PricingProducer {
    pub async fn run(config: Arc<Config>) -> Result<()> {
        let interval_in_seconds = 1 * 60; // TODO make this longer
        let mut interval = time::interval(Duration::from_secs(interval_in_seconds));

        // let watchlists = watchlist::get_all(&config);
        // // TEMP Initialization of a watchlist
        // if watchlists.len() == 0 {
        //     watchlist::insert(
        //         &config,
        //         Location {
        //             id: 1645,
        //             level: 4,
        //             name: String::from("Ullanlinna"),
        //         },
        //         1,
        //         Some(2.0),
        //     );
        // }

        loop {
            info!("Starting PricingProducer run");
            let start = Instant::now();

            let watchlists = watchlist::get_all(&config);

            for watchlist in watchlists {
                info!(
                    "Starting calculating prices for watchlist_id: {:?}",
                    watchlist.id
                );

                let mut oikotie_client = Oikotie::new().await;

                let apartments: Vec<InsertableApartment> =
                    oikotie_client.get_apartments(&watchlist).await.unwrap();

                process_apartment_calculations(&config, apartments, oikotie_client).await?;

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
