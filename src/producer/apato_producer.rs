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

/// Creates and sends UpdateWatchlist tasks to the message queue for each watchlist..
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

/// Creates and sends SendMessage tasks to the message queue for each watchlist.
async fn handle_update_message_tasks(
    config: &Arc<Config>,
    bot: Arc<Bot>,
    producer_sender: Sender<MessageTask>,
) {
    let watchlists = db::watchlist::get_all(config);

    for watchlist in watchlists {
        let chat_id = watchlist.chat_id;

        match find_apartments_to_send(config, watchlist.clone(), chat_id, bot.clone()).await {
            // TODO FIX
            Ok(apartments) => {
                for ap in apartments {
                    let task = MessageTask {
                        task_type: TaskType::SendMessage,
                        watchlist: watchlist.clone(),
                        apartment: Some(ap),
                    };
                    // TODO fix
                    let _ = producer_sender.send(task).await;
                }
            }
            Err(_e) => {}
        }
    }
}

/// Finds apartments from given watchlist that matches criteria and has not been sent.
async fn find_apartments_to_send(
    config: &Arc<Config>,
    watchlist: Watchlist,
    chat_id: i64,
    bot: Arc<Bot>,
) -> Result<Vec<Apartment>> {
    let unsent_apartments =
        db::apartment_watchlist::get_unsent_apartments(config, &watchlist.clone());

    let new_targets = match unsent_apartments {
        Ok(v) => v,
        Err(e) => {
            error!("Consumer Error while fetching new targets: {:?}", e);
            bot.send_message(ChatId(chat_id), format!("{}", e)).await?;
            return Ok(Vec::new());
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
    Ok(aps)
}
