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

            let watchlists = watchlist::get_all(config);

            for watchlist in watchlists {
                info!(
                    "Starting calculating prices for watchlist_id: {:?}",
                    watchlist.id
                );

                let mut oikotie_client = Oikotie::new().await;

                let now = Instant::now();
                let apartments: Vec<InsertableApartment> = oikotie_client
                    .get_apartments(config.clone(), &watchlist)
                    .await
                    .unwrap_or_default();

                let duration_process = now.elapsed();
                info!("OIKOTIE PROCESS TAKES {:?}", duration_process);

                let now_ = Instant::now();
                process_apartment_calculations(config, apartments, oikotie_client).await?;
                let duration_process_ = now_.elapsed();

                info!(
                    "APARTMENT CALCULATION PROCESS TAKES {:?}",
                    duration_process_
                );

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
