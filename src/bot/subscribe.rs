use std::sync::Arc;

use log::error;
use teloxide::{prelude::Requester, types::ChatId, Bot};

use crate::{config::Config, services::watchlists};
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
    match watchlists::subscribe(
        config.clone(),
        chat_id.0,
        location.clone(),
        size,
        new_target_yield,
    )
    .await
    {
        Ok(watchlist) => {
            tg.send_message(
                chat_id,
                format!(
                    "Added watchlist {} ({}) with target yield {:.2}%",
                    watchlist.id, watchlist.location_name, new_target_yield
                ),
            )
            .await?;
        }
        Err(err) => {
            error!("Error while subscribing: {}", err);
            tg.send_message(
                chat_id,
                "Could not subscribe. Please check the details and try again.",
            )
            .await?;
        }
    }

    Ok(())
}
