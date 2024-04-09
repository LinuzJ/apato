use anyhow::Result;
use chrono::{Duration, Utc};
use log::{error, info};
use std::{
    ops::Sub,
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
    db::{apartment::get_new_for_watchlist, watchlist::get_all},
    models::watchlist::Watchlist,
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

                check_watchlist_for_new_apartment(config, watchlist, chat_id, bot).await?;
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

async fn check_watchlist_for_new_apartment(
    config: &Arc<Config>,
    watchlist: Watchlist,
    chat_id: i64,
    bot: Arc<Bot>,
) -> Result<()> {
    let now = Utc::now().naive_local();
    let last_period_end = now.sub(Duration::seconds(config.consumer_timeout_seconds.into()));
    let potential_apartments = get_new_for_watchlist(config, watchlist.clone(), last_period_end);

    let new_targets = match potential_apartments {
        Ok(v) => v,
        Err(e) => {
            error!("Consumer Error while fetching new targets: {:?}", e);
            bot.send_message(ChatId(chat_id), format!("{}", e.to_string()))
                .await?;
            return Ok(());
        }
    };

    if new_targets.len() > 1 {
        let watchlist_clone = watchlist.clone();
        bot.send_message(
            ChatId(chat_id),
            format!(
                "Found new apartments for your watchlist {} for {}",
                watchlist_clone.id, watchlist_clone.location_name
            ),
        )
        .await?;

        let formatted: Vec<String> = new_targets
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
    } else if new_targets.len() == 1 {
        let watchlist_clone = watchlist.clone();
        bot.send_message(
            ChatId(chat_id),
            format!(
                "Found a new apartment for your watchlist {} for {}",
                watchlist_clone.id, watchlist_clone.location_name
            ),
        )
        .await?;

        let formatted = format_apartment_message(&new_targets[0]);

        bot.send_message(ChatId(chat_id), formatted).await?;
    }

    Ok(())
}
