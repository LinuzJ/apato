use crate::{
    config::Config,
    db::{self, watchlist},
    models::{
        apartment::InsertableApartment,
        watchlist::{SizeTarget, Watchlist},
    },
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
use tokio::sync::{broadcast::Receiver, watch};

pub struct Producer;

impl Producer {
    pub async fn run(
        config: &Arc<Config>,
        shutdown: Arc<AtomicBool>,
        mut shutdown_rx: Receiver<()>,
    ) -> Result<()> {
        let interval_in_seconds = config.producer_timeout_seconds as u64; // TODO make this longer
        let interval = Duration::from_secs(interval_in_seconds);

        // Main Producer loop
        while !shutdown.load(Ordering::Acquire) {
            info!("Starting PricingProducer run");
            let start = Instant::now();

            let watchlists = watchlist::get_all(config);
            let mut watchlist_handles = Vec::new();

            for watchlist in watchlists {
                let config_clone = config.clone();
                let handle = tokio::task::spawn(async move {
                    handle_watchlist_producer(&config_clone, &watchlist).await;
                });
                watchlist_handles.push(handle);
            }

            // TODO: Make this wait in a non-sequential way?
            for handle in watchlist_handles {
                handle.await?;
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

async fn handle_watchlist_producer(config: &Arc<Config>, watchlist: &Watchlist) {
    info!("Starting producer run for watchlist_id: {:?}", watchlist.id);

    let mut oikotie_client = Oikotie::new().await;

    let mut target_size = SizeTarget::empty();
    if let Some(min_size) = watchlist.target_size_min {
        target_size.min = Some(min_size)
    }
    if let Some(max_size) = watchlist.target_size_max {
        target_size.max = Some(max_size)
    }

    let now = Instant::now();
    let apartments: Vec<InsertableApartment> = oikotie_client
        .get_apartments(config.clone(), &watchlist, target_size)
        .await
        .unwrap_or_default();

    let duration_process = now.elapsed();
    info!(
        "Producer oikotie fetch section took {:?} for watchlist {:?}",
        duration_process, watchlist.id
    );

    let now_ = Instant::now();
    let mut apartment_handles = Vec::new();

    for apartment in apartments {
        let oiko_clone = oikotie_client.clone();
        let watchlist_clone = watchlist.clone();
        let config_clone = config.clone();
        let handle = tokio::task::spawn(async move {
            handle_apartmet_producer(&config_clone, oiko_clone, apartment, watchlist_clone).await;
        });

        apartment_handles.push(handle);
    }

    for handle in apartment_handles {
        let _ = handle.await;
    }
    let duration_process_ = now_.elapsed();

    info!(
        "Producer apartment section took {:?} for watchlist {:?}",
        duration_process_, watchlist.id
    );

    info!("Finished producer for watchlist_id: {:?}", watchlist.id);
}

async fn handle_apartmet_producer(
    config: &Arc<Config>,
    oikotie: Oikotie,
    apartment: InsertableApartment,
    watchlist: Watchlist,
) {
    let complete_apartment = process_apartment_calculations(config, apartment, oikotie).await;

    match complete_apartment {
        Ok(ap) => {
            // Insert into apartment table
            db::apartment::insert(config, ap.clone());

            // Add to watchlist index if over target yield
            if ap.estimated_yield.unwrap_or_default() > watchlist.target_yield.unwrap_or_default() {
                db::watchlist_apartment_index::insert(config, watchlist.id, ap.card_id);
            }
        }
        Err(e) => {
            error!("Producer Error: While processing calculations {}", e);
        }
    }
}
