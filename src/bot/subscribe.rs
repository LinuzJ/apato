use std::sync::Arc;

use log::info;
use teloxide::{prelude::Requester, types::ChatId, Bot};

use crate::{
    config::Config,
    db,
    models::watchlist::SizeTarget,
    oikotie::oikotie::{Location, Oikotie},
};
use anyhow::Result;

use super::bot_types::SubscriptionArgs;

pub fn check_args(args: SubscriptionArgs) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    if args.location.is_empty()
        || args.target_yield.is_none()
        || args.min_size.is_none()
        || args.max_size.is_none()
    {
        errors.push(String::from("Missing value in args."));
    }

    errors
}

pub async fn subscribe_to_watchlist(
    size: (f64, f64),
    new_target_yield: f64,
    location: String,
    chat_id: ChatId,
    tg: &Bot,
    config: Arc<Config>,
) -> Result<()> {
    // Check if watchlist for this location already exists
    let existing = db::watchlist::get_for_chat_and_location(&config, chat_id.0, &location);

    if !existing.is_empty() {
        tg.send_message(
            chat_id,
            "You already have a watchlist for this location. Updating target yield...",
        )
        .await?;

        match db::watchlist::update_yield(&config, existing[0].id, new_target_yield).await {
            Ok(()) => {
                info!("Updated yield for watchlist {}", existing[0].id);
            }
            Err(e) => {
                tg.send_message(chat_id, format!("Error while updating yield: {}", e))
                    .await?;
            }
        };

        return Ok(());
    }

    // Create new watchlist
    let mut oikotie_client: Oikotie = Oikotie::new().await;
    let location_id_response = oikotie_client.get_location_id(&location).await;
    let mut watchlist_location: Option<Location> = None;

    match location_id_response {
        Ok(location_id) => {
            watchlist_location = Some(Location {
                id: location_id as i32,
                level: 4, // TODO maybe not just hardcode this
                name: location,
            })
        }
        Err(e) => {
            let err_str = e.to_string();
            tg.send_message(chat_id, err_str).await?;
        }
    }

    let (min_size, max_size) = size;
    let mut target_size = SizeTarget::empty();
    target_size.min = Some(min_size as i32);
    target_size.max = Some(max_size as i32);

    if let Some(loc) = watchlist_location {
        db::watchlist::insert(&config, loc, chat_id.0, Some(new_target_yield), target_size);
        tg.send_message(chat_id, "Added to your watchlist!").await?;
    }

    Ok(())
}