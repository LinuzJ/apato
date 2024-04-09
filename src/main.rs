extern crate chrono;
extern crate diesel;
extern crate tokio;

mod bot;
mod config;
mod consumer;
mod db;
mod interest_rate;
mod logger;
mod models;
mod oikotie;
mod producer;

use anyhow::Result;
use bot::bot::ApatoTelegramBot;
use config::Config;
use consumer::apato_consumer::Consumer;
use log::{error, info};
use logger::setup_logger;
use producer::apato_producer::Producer;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let config: Arc<Config> = Arc::new(config::read_config());

    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
    let shutdown_rx_2 = shutdown_tx.subscribe();
    let shutdown = Arc::new(AtomicBool::new(false));

    let bot = ApatoTelegramBot::new(config.clone()).await?;

    let producer_handle = {
        let shutdown = shutdown.clone();
        let config_clone = config.clone();
        tokio::task::spawn(async move { Producer::run(&config_clone, shutdown, shutdown_rx).await })
    };

    let consumer_handle = {
        let shutdown = shutdown.clone();
        let tg_bot = bot.tg.clone();
        let config_clone = config.clone();
        tokio::task::spawn(async move {
            Consumer::run(&config_clone, shutdown, shutdown_rx_2, tg_bot).await
        })
    };

    let (bot_handle, bot_shutdown_token) = bot.spawn();

    {
        let shutdown = shutdown.clone();
        std::thread::spawn(move || {
            let mut forward_signals =
                Signals::new([SIGINT, SIGTERM]).expect("unable to watch for signals");

            for signal in forward_signals.forever() {
                info!("Shutting down... Recieved {signal}");

                // Shut down Producer and Consumer
                shutdown.swap(true, Ordering::Relaxed);

                // Shut down Telegram bot
                let _res = bot_shutdown_token.shutdown();

                // Broadcast shutdown
                let _res = shutdown_tx.send(()).unwrap_or_else(|_| {
                    std::process::exit(0);
                });
            }
        });
    }

    if let Err(err) = tokio::try_join!(producer_handle, consumer_handle, bot_handle) {
        error!("Error: {:?}", err)
    }

    Ok(())
}
