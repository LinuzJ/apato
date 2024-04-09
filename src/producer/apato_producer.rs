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
        let interval_in_seconds = config.producer_timeout_seconds as u64; // TODO make this longer
        let interval = Duration::from_secs(interval_in_seconds);

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
               _ = tokio::time::sleep(interval) => {}
               _ = shutdown_rx.recv() => {
                   break
               }
            }
        }
        Ok(())
    }
}
