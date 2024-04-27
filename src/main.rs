extern crate chrono;
extern crate diesel;
extern crate tokio;

use anyhow::Result;

use apato::{
    bot::{bot::ApatoTelegramBot},
    config::{self, Config},
    consumer::{apato_consumer::Consumer},
    logger::{setup_logger},
    producer::{apato_producer::Producer},
    MessageTask,
};
use futures::future::TryJoinAll;
use log::{error, info};
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
    let consumer_amount = 6;

    let (producer_sender, consumer_reciever) = async_channel::unbounded::<MessageTask>();

    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
    let shutdown = Arc::new(AtomicBool::new(false));

    let bot = ApatoTelegramBot::new(config.clone()).await?;

    let producer_handle = {
        let shutdown = shutdown.clone();
        let config = config.clone();
        let tg_bot = bot.tg.clone();

        tokio::task::spawn(async move {
            Producer::run(&config, shutdown, producer_sender, tg_bot, shutdown_rx).await
        })
    };

    let mut consumer_handles = Vec::new();
    for consumer in 0..consumer_amount {
        let consumer_handle = {
            let shutdown = shutdown.clone();
            let tg_bot = bot.tg.clone();
            let config_clone = config.clone();
            let shutdown_rx_clone = shutdown_tx.subscribe();
            let consumer_reciever = consumer_reciever.clone();
            tokio::task::spawn(async move {
                Consumer::run(
                    &config_clone,
                    consumer_reciever,
                    shutdown,
                    shutdown_rx_clone,
                    tg_bot,
                    consumer,
                )
                .await
            })
        };
        consumer_handles.push(consumer_handle)
    }

    let (bot_handle, bot_shutdown_token) = bot.spawn();

    {
        let shutdown = shutdown.clone();
        std::thread::spawn(move || {
            let mut forward_signals =
                Signals::new([SIGINT, SIGTERM]).expect("unable to watch for signals");

            for signal in forward_signals.forever() {
                info!("Shutting down... Recieved {signal}");

                shutdown.swap(true, Ordering::Relaxed);

                let _res = bot_shutdown_token.shutdown();

                let _res = shutdown_tx.send(()).unwrap_or_else(|_| {
                    std::process::exit(0);
                });
            }
        });
    }

    let join_consumer_handles = consumer_handles.into_iter().collect::<TryJoinAll<_>>();

    if let Err(err) = tokio::try_join!(producer_handle, join_consumer_handles, bot_handle,) {
        error!("Error: {:?}", err)
    }

    Ok(())
}
