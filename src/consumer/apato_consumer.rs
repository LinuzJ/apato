use anyhow::Result;
use async_channel::Receiver;
use futures::future::TryJoinAll;
use log::{error, info};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio::sync::broadcast;

use crate::{
    bot::bot::format_apartment_message,
    config::Config,
    db::{self},
    models::{
        apartment::{Apartment, InsertableApartment},
        watchlist::{SizeTarget, Watchlist},
    },
    oikotie::oikotie::Oikotie,
    producer::calculations::process_apartment_calculations,
    MessageTask, TaskType,
};

pub struct Consumer;

impl Consumer {
    pub async fn run(
        config: &Arc<Config>,
        consumer_reciever: Receiver<MessageTask>,
        shutdown: Arc<AtomicBool>,
        mut shutdown_rx: broadcast::Receiver<()>,
        bot: Arc<Bot>,
        consumer_number: i32,
    ) -> Result<()> {
        let interval_in_seconds = config.consumer_timeout_seconds.into();
        let interval = std::time::Duration::from_secs(interval_in_seconds);

        while !shutdown.load(Ordering::Acquire) {
            let queue_message = consumer_reciever.try_recv();

            if let Ok(task) = queue_message {
                let start = Instant::now();
                info!("Starting Consumer run");
                let bot = bot.clone();
                // TODO Handle errors from both
                match task.task_type {
                    TaskType::UpdateWatchlist => {
                        update_watchlist_task(config, task.watchlist, consumer_number).await
                    }
                    TaskType::SendMessage => {
                        send_message_task(config, task.watchlist, task.apartment.unwrap(), bot)
                            .await?
                    }
                }
                let duration = start.elapsed();
                info!("Finished Consumer run in {:?}", duration);
            }

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

async fn send_message_task(
    config: &Arc<Config>,
    watchlist: Watchlist,
    apartment: Apartment,
    bot: Arc<Bot>,
) -> Result<()> {
    let chat_id = watchlist.chat_id;
    let ap = db::apartment::get_card_id(config, apartment.card_id);

    if let Ok(unsent_ap) = ap {
        let formatted = format_apartment_message(&watchlist, &unsent_ap[0]);
        bot.send_message(ChatId(chat_id), formatted).await?;

        db::watchlist_apartment_index::set_to_read(config, &watchlist, unsent_ap[0].card_id);
    }

    Ok(())
}

async fn update_watchlist_task(config: &Arc<Config>, watchlist: Watchlist, consumer_number: i32) {
    info!(
        "Starting watchlist task run for watchlist_id: {:?} on consumer {}",
        watchlist.id, consumer_number
    );

    let mut oikotie_client = Oikotie::new().await;

    let target_size = get_target_size(watchlist.target_size_min, watchlist.target_size_max);

    // TODO make this faster ?
    let apartments: Vec<InsertableApartment> = oikotie_client
        .get_apartments(config.clone(), &watchlist, target_size)
        .await
        .unwrap_or_default();

    // TODO Use thread pool
    let mut apartment_handles = Vec::new();
    for apartment in apartments {
        let oiko_clone = oikotie_client.clone();
        let watchlist_clone = watchlist.clone();
        let config_clone = config.clone();
        let handle = tokio::task::spawn(async move {
            process_apartment(
                &config_clone,
                oiko_clone,
                apartment,
                watchlist_clone,
                consumer_number,
            )
            .await;
        });

        apartment_handles.push(handle);
    }

    let join_apartment_handles = apartment_handles.into_iter().collect::<TryJoinAll<_>>();

    if let Err(err) = tokio::try_join!(join_apartment_handles) {
        error!("Error: {:?}", err)
    }

    info!(
        "Finished watchlist task for watchlist_id: {:?} on consumer {}",
        watchlist.id, consumer_number
    );
}

async fn process_apartment(
    config: &Arc<Config>,
    oikotie: Oikotie,
    apartment: InsertableApartment,
    watchlist: Watchlist,
    consumer_number: i32,
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
            error!(
                "Consumer Error: While processing calculations {} on consumer {}",
                e, consumer_number
            );
        }
    }
}

fn get_target_size(min: Option<i32>, max: Option<i32>) -> SizeTarget {
    let mut target_size = SizeTarget::empty();
    if let Some(min_size) = min {
        target_size.min = Some(min_size)
    }
    if let Some(max_size) = max {
        target_size.max = Some(max_size)
    }
    target_size
}
