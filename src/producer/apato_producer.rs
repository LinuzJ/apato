use crate::{
    config::Config,
    db::{self, watchlist},
    models::{apartment::InsertableApartment, watchlist::SizeTarget},
    oikotie::oikotie::Oikotie,
    producer::calculations::process_apartment_calculations,
};
use anyhow::Result;
use log::{error, info};
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

                let mut target_size = SizeTarget::empty();
                if let Some(min_size) = watchlist.target_size_min {
                    target_size.min = Some(min_size as i32)
                }
                if let Some(max_size) = watchlist.target_size_max {
                    target_size.max = Some(max_size as i32)
                }

                let now = Instant::now();
                let apartments: Vec<InsertableApartment> = oikotie_client
                    .get_apartments(config.clone(), &watchlist, target_size)
                    .await
                    .unwrap_or_default();

                let duration_process = now.elapsed();
                info!("OIKOTIE PROCESS TAKES {:?}", duration_process);

                let now_ = Instant::now();
                for apartment in apartments {
                    let oiko_clone = oikotie_client.clone();
                    let complete_apartment =
                        process_apartment_calculations(config, apartment, oiko_clone).await;

                    match complete_apartment {
                        Ok(ap) => {
                            // Insert into apartment table
                            db::apartment::insert(config, ap.clone());

                            // Add to watchlist index if over target yield
                            if ap.estimated_yield.unwrap_or_default()
                                > watchlist.target_yield.unwrap_or_default()
                            {
                                db::watchlist_apartment_index::insert(
                                    config,
                                    watchlist.id,
                                    ap.card_id,
                                );
                            }
                        }
                        Err(e) => {
                            error!("Producer Error: While processing calculations {}", e);
                        }
                    }
                }
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
