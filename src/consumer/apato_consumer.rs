use anyhow::Result;
use log::{error, info};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio::sync::broadcast::Receiver;

use crate::{
    bot::bot::format_apartment_message,
    config::Config,
    db::{self, watchlist::get_all},
    models::{apartment::Apartment, watchlist::Watchlist},
};

pub struct Consumer;

impl Consumer {
    pub async fn run(
        config: &Arc<Config>,
        shutdown: Arc<AtomicBool>,
        mut shutdown_rx: Receiver<()>,
        bot: Arc<Bot>,
    ) -> Result<()> {
        let interval_in_seconds = config.consumer_timeout_seconds.into();
        let interval = std::time::Duration::from_secs(interval_in_seconds);

        while !shutdown.load(Ordering::Acquire) {
            info!("Starting Consumer run");
            let start = Instant::now();

            // For each watchlist
            let watchlists = get_all(config);

            for watchlist in watchlists {
                let bot = bot.clone();
                let chat_id = watchlist.chat_id;

                check_for_new_apartments(config, watchlist, chat_id, bot).await?;
            }

            let duration = start.elapsed();
            info!("Finished Consumer run in {:?}", duration);

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

async fn check_for_new_apartments(
    config: &Arc<Config>,
    watchlist: Watchlist,
    chat_id: i64,
    bot: Arc<Bot>,
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

    let amount_of_matches = new_targets.len();

    match amount_of_matches.cmp(&1) {
        std::cmp::Ordering::Greater => {
            let mut aps: Vec<Apartment> = vec![];

            for card_id in new_targets {
                let ap = db::apartment::get_card_id(config, card_id);
                if let Ok(unsent_ap) = ap {
                    let ap_clone = unsent_ap[0].clone();
                    aps.push(ap_clone);
                }
            }

            let watchlist_clone = watchlist.clone();
            bot.send_message(
                ChatId(chat_id),
                format!(
                    "Found new apartments for your watchlist {} for {}",
                    watchlist_clone.id, watchlist_clone.location_name
                ),
            )
            .await?;

            let formatted: Vec<String> = aps
                .iter()
                .enumerate()
                .map(|(index, apartment)| {
                    let formatted = format_apartment_message(apartment);
                    format!("{}: \n {}", index, formatted)
                })
                .collect();

            for message_to_send in formatted {
                bot.send_message(ChatId(chat_id), message_to_send).await?;
            }

            for ap in aps {
                db::watchlist_apartment_index::set_to_read(config, &watchlist, ap.card_id);
            }
        }
        std::cmp::Ordering::Equal => {
            let watchlist_clone = watchlist.clone();
            bot.send_message(
                ChatId(chat_id),
                format!(
                    "Found a new apartment for your watchlist {} for {}",
                    watchlist_clone.id, watchlist_clone.location_name
                ),
            )
            .await?;

            let ap = db::apartment::get_card_id(config, new_targets[0]);

            if let Ok(unsent_ap) = ap {
                let formatted = format_apartment_message(&unsent_ap[0]);
                bot.send_message(ChatId(chat_id), formatted).await?;

                db::watchlist_apartment_index::set_to_read(
                    config,
                    &watchlist,
                    unsent_ap[0].card_id,
                );
            }
        }
        std::cmp::Ordering::Less => {
            info!("Consumer found nothing to send...")
        }
    }

    Ok(())
}
