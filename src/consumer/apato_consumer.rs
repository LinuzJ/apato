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
use teloxide::Bot;
use tokio::sync::broadcast::Receiver;
use tokio::time;

pub struct Consumer;

impl Consumer {
    pub async fn run(
        config: &Arc<Config>,
        shutdown: Arc<AtomicBool>,
        mut shutdown_rx: Receiver<()>,
        bot: Arc<Bot>,
    ) -> Result<()> {
        let interval_in_seconds = config.consumer_timeout_seconds as u64; // TODO make this longer
        let mut interval = time::interval(Duration::from_secs(interval_in_seconds));

        while !shutdown.load(Ordering::Acquire) {
            info!("Starting Consumer run");
            let start = Instant::now();

            println!("CONSUMER");

            let duration = start.elapsed();
            info!("Finished Consumer run in {:?}", duration);

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
