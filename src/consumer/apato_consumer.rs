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
use tokio::sync::{broadcast, Semaphore};

use crate::{
    bot::bot::format_apartment_message,
    config::Config,
    db,
    models::{
        apartment::{Apartment, InsertableApartment},
        watchlist::{SizeTarget, Watchlist},
    },
    oikotie::oikotie::Oikotie,
    MessageTask, TaskType,
};

use super::calculations::get_estimated_irr;

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
                info!("Starting run on Consumer {}", consumer_number);
                let bot = bot.clone();
                // TODO Handle errors from both
                match task.task_type {
                    TaskType::UpdateWatchlist => {
                        let result =
                            update_watchlist_task(config, task.watchlist, consumer_number).await;

                        match result {
                            Ok(ok) => ok,
                            Err(e) => error!("Error in Consumer {}: {:?}", consumer_number, e),
                        }
                    }
                    TaskType::SendMessage => {
                        send_message_task(config, task.watchlist, task.apartment.unwrap(), bot)
                            .await?
                    }
                }
                let duration = start.elapsed();
                info!(
                    "Finished run in {:?} seconds on consumer {}",
                    duration, consumer_number
                );
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
    let apatment_result = db::apartment::get_apartment_by_card_id(config, apartment.card_id);
    match apatment_result {
        Ok(a) => {
            if let Some(ap) = a {
                let formatted = format_apartment_message(&watchlist, &ap);
                bot.send_message(ChatId(chat_id), formatted).await?;

                db::apartment_watchlist::set_to_read(config, &watchlist, apartment.card_id);
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

/// Updates the given watchist.
///
/// 1) Fetches apartments from Oikotie
/// 2)
///
/// # Examples
///
/// ```ignore
/// // You can have rust code between fences inside the comments
/// // If you pass --test to `rustdoc`, it will even test it for you!
/// use doc::Person;
/// let person = Person::new("name");
/// ```
async fn update_watchlist_task(
    config: &Arc<Config>,
    watchlist: Watchlist,
    consumer_number: i32,
) -> Result<()> {
    info!(
        "Starting watchlist task run for watchlist_id: {:?} on consumer {}",
        watchlist.id, consumer_number
    );

    let mut oikotie_client = Oikotie::new().await;

    let target_size = get_target_size(watchlist.target_size_min, watchlist.target_size_max);

    // TODO make this faster ?
    let apartments: Vec<InsertableApartment> = match oikotie_client
        .get_apartments(config.clone(), &watchlist, target_size)
        .await
    {
        Ok(aps) => aps,
        Err(e) => return Err(e),
    };

    // Cap the amount of apartments processed at the same time
    let sem = Arc::new(Semaphore::new(
        usize::try_from(config.consumer_thread_limit).unwrap(),
    ));

    let mut apartment_handles = Vec::new();
    for apartment in apartments {
        let permit = Arc::clone(&sem).acquire_owned().await;

        let oiko_clone = oikotie_client.clone();
        let watchlist_clone = watchlist.clone();
        let config_clone = config.clone();

        let handle = tokio::task::spawn(async move {
            let _permit = permit;
            match process_apartment(
                &config_clone,
                oiko_clone,
                apartment,
                watchlist_clone,
                consumer_number,
            )
            .await
            {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        });

        apartment_handles.push(handle);
    }

    let join_apartment_handles = apartment_handles.into_iter().collect::<TryJoinAll<_>>();

    if let Err(err) = tokio::try_join!(join_apartment_handles) {
        return Err(err.into());
    }

    info!(
        "Finished watchlist task for watchlist_id: {:?} on consumer {}",
        watchlist.id, consumer_number
    );

    Ok(())
}

/// Process one apartment.
///
/// Checks if apartment already exists in database
///     If yes:
///         Is it fresh?
///             If not:
///                 Recalculate Yield
///         Has it already been added to the target index?
///             If not:
///                 Add
///     If not:
///         Calculate yield and add to target index
async fn process_apartment(
    config: &Arc<Config>,
    mut oikotie: Oikotie,
    mut apartment: InsertableApartment,
    watchlist: Watchlist,
    consumer_number: i32,
) -> Result<()> {
    // Check if apartment already exists in db
    let apartment_from_db_res = db::apartment::get_apartment_by_card_id(config, apartment.card_id);

    let apartment_from_db = match apartment_from_db_res {
        Ok(aps) => aps,
        Err(e) => return Err(e.into()),
    };

    if let Some(mut existing_apartment) = apartment_from_db {
        // Check if the entry is fresh
        let is_fresh = match db::apartment::apartment_is_fresh(config, apartment.card_id) {
            Ok(b) => b,
            Err(e) => return Err(e.into()),
        };

        if !is_fresh {
            let estimated_rent = oikotie.get_estimated_rent(&apartment).await?;
            apartment.rent = Some(estimated_rent);
            let new_irr = match get_estimated_irr(config, apartment.clone()).await {
                Ok(irr) => irr,
                Err(e) => return Err(e),
            };

            db::apartment::update_yield(config, apartment.card_id, new_irr);
            existing_apartment.estimated_yield = Some(new_irr);
        }

        // Check if this aparment existst in target apartments
        let index_exists =
            match db::apartment_watchlist::exists(config, watchlist.id, apartment.card_id) {
                Ok(exists) => exists,
                Err(e) => return Err(e.into()),
            };

        if !index_exists {
            // Add to watchlist index if over target yield
            if existing_apartment.estimated_yield.unwrap_or_default()
                > watchlist.target_yield.unwrap_or_default()
            {
                db::apartment_watchlist::insert(config, watchlist.id, apartment.card_id);
            }
        }
    } else {
        let estimated_rent = oikotie.get_estimated_rent(&apartment).await?;

        apartment.rent = Some(estimated_rent);

        match get_estimated_irr(config, apartment.clone()).await {
            Ok(irr) => {
                apartment.estimated_yield = Some(irr);

                // Insert into apartment table
                db::apartment::insert(config, apartment.clone());

                // Add to watchlist index if over target yield
                if apartment.estimated_yield.unwrap_or_default()
                    > watchlist.target_yield.unwrap_or_default()
                {
                    db::apartment_watchlist::insert(config, watchlist.id, apartment.card_id);
                }
            }
            Err(e) => {
                error!(
                    "Consumer Error: While processing calculations on consumer {}: {}",
                    consumer_number, e
                );
                return Err(e);
            }
        };
    }

    Ok(())
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
