use crate::{
    config::Config,
    db::{self, watchlist},
    models::{apartment::Apartment, watchlist::Watchlist},
    MessageTask, TaskType,
};
use anyhow::Result;
use async_channel::Sender;
use log::error;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio::sync::broadcast::Receiver;

pub struct Producer;

impl Producer {
    pub async fn run(
        config: &Arc<Config>,
        shutdown: Arc<AtomicBool>,
        producer_sender: Sender<MessageTask>,
        bot: Arc<Bot>,
        mut shutdown_rx: Receiver<()>,
    ) -> Result<()> {
        let interval_in_seconds = config.producer_timeout_seconds as u64;
        let interval = Duration::from_secs(interval_in_seconds);

        while !shutdown.load(Ordering::Acquire) {
            // TODO handle errors
            handle_watchlists_tasks(config, producer_sender.clone()).await;

            handle_update_message_tasks(config, bot.clone(), producer_sender.clone()).await;

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

async fn handle_watchlists_tasks(config: &Arc<Config>, producer_sender: Sender<MessageTask>) {
    let watchlists = watchlist::get_all(&config.clone());
    for watchlist in watchlists {
        let _ = producer_sender
            .send(MessageTask {
                task_type: TaskType::UpdateWatchlist,
                watchlist,
                apartment: None,
            })
            .await;
    }
}

async fn handle_update_message_tasks(
    config: &Arc<Config>,
    bot: Arc<Bot>,
    producer_sender: Sender<MessageTask>,
) {
    let watchlists = db::watchlist::get_all(config);

    for watchlist in watchlists {
        let chat_id = watchlist.chat_id;

        match check_for_new_apartments_to_send(
            config,
            watchlist,
            chat_id,
            bot.clone(),
            producer_sender.clone(),
        )
        .await
        {
            // TODO FIX
            Ok(_) => {}
            Err(_e) => {}
        }
    }
}

async fn check_for_new_apartments_to_send(
    config: &Arc<Config>,
    watchlist: Watchlist,
    chat_id: i64,
    bot: Arc<Bot>,
    producer_sender: Sender<MessageTask>,
) -> Result<()> {
    let unsent_apartments =
        db::watchlist_apartment_index::get_unsent_apartments(config, &watchlist.clone());

    let new_targets = match unsent_apartments {
        Ok(v) => v,
        Err(e) => {
            error!("Consumer Error while fetching new targets: {:?}", e);
            bot.send_message(ChatId(chat_id), format!("{}", e)).await?;
            return Ok(());
        }
    };

    let mut aps: Vec<Apartment> = Vec::new();

    for card_id in new_targets {
        let ap = db::apartment::get_apartment_by_card_id(config, card_id);
        match ap {
            Ok(a) => {
                if let Some(apartment) = a {
                    let ap_clone = apartment.clone();
                    aps.push(ap_clone);
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    for ap in aps {
        let task = MessageTask {
            task_type: TaskType::SendMessage,
            watchlist: watchlist.clone(),
            apartment: Some(ap),
        };
        // TODO fix
        let _ = producer_sender.send(task).await;
    }
    Ok(())
}
