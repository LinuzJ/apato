use crate::{
    config::Config, db::watchlist, models::apartment::InsertableApartment,
    oikotie::oikotie::Oikotie, producer::calculations::process_apartment_calculations,
};
use anyhow::Result;
use log::info;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::broadcast::Receiver;
use tokio::time;

pub struct Producer;

impl Producer {
    pub async fn run(
        config: &Arc<Config>,
        shutdown: Arc<AtomicBool>,
        mut shutdown_rx: Receiver<()>,
    ) -> Result<()> {
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
        while !shutdown.load(Ordering::Acquire) {
            info!("Starting PricingProducer run");
            let start = Instant::now();

            let watchlists = watchlist::get_all(&config);

            for watchlist in watchlists {
                info!(
                    "Starting calculating prices for watchlist_id: {:?}",
                    watchlist.id
                );

                let mut oikotie_client = Oikotie::new().await;

                let apartments: Vec<InsertableApartment> = oikotie_client
                    .get_apartments(&watchlist)
                    .await
                    .unwrap_or_default();

                process_apartment_calculations(&config, apartments, oikotie_client).await?;

                info!(
                    "Finished price calculations for watchlist_id: {:?}",
                    watchlist.id
                );
            }

            let duration = start.elapsed();
            info!("Finished PricingProducer run in {:?}", duration);

            tokio::select! {
               _ = interval.tick() => {}
               _ = shutdown_rx.recv() => {
                   break
               }
            }
        }
        Ok(())
    }
}
