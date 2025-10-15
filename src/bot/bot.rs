use crate::{
    bot::subscribe::{check_args, subscribe_to_watchlist},
    config::Config,
    db::{self},
    models::{apartment::Apartment, watchlist::Watchlist},
};
use anyhow::Result;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use std::sync::Arc;
use teloxide::{
    dispatching::{DefaultKey, HandlerExt, UpdateFilterExt},
    dptree,
    prelude::{Dispatcher, LoggingErrorHandler},
    requests::Requester,
    types::{Message, Update},
    utils::command::{BotCommands, ParseError},
    Bot,
};

use super::bot_types::SubscriptionArgs;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Help")]
    Help,

    #[command(
        description = "Subscribe to a location watchlist. Provide the args in the following format: < /sub {location name} min_size={size (m^2)} max_size={size (m^2)} yield={target yield}. > \n\n Example: \n '< /sub ullanlinna min_size=50 max_size=60 yield=10 >",
        parse_with = parse_subscribe_message
    )]
    Sub(SubscriptionArgs),

    #[command(
        description = "Unsubscribe to a watchlist. Use watchlist ID.",
        parse_with = parse_string_to_int_message
    )]
    Unsub(Option<i32>),

    #[command(description = "List current active watchlist subscriptions")]
    ListWatchlists,

    #[command(description = "Get all apartments/houses in watchlist",
        parse_with = parse_string_to_int_message
    )]
    GetAll(Option<i32>),

    #[command(
        description = "Get all apartments/houses in watchlist that are above or equal to the target yield",
        parse_with = parse_string_to_int_message
    )]
    GetMatching(Option<i32>),
}

pub struct ApatoTelegramBot {
    pub dispatcher: Dispatcher<Arc<Bot>, anyhow::Error, DefaultKey>,
    pub tg: Arc<Bot>,
}

impl ApatoTelegramBot {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let telegram_bot_token = &config.telegram_bot_token;

        let tg = Arc::new(Bot::new(telegram_bot_token));
        tg.set_my_commands(Command::bot_commands()).await?;

        let handler = Update::filter_message().branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_command),
        );

        let dispatcher = Dispatcher::builder(tg.clone(), handler)
            .dependencies(dptree::deps![config])
            .error_handler(LoggingErrorHandler::with_custom_text(
                "an error has occurred in the dispatcher",
            ))
            .build();

        let bot = ApatoTelegramBot {
            dispatcher,
            tg: tg.clone(),
        };

        Ok(bot)
    }

    pub fn spawn(
        mut self,
    ) -> (
        tokio::task::JoinHandle<()>,
        teloxide::dispatching::ShutdownToken,
    ) {
        let shutdown_token = self.dispatcher.shutdown_token();
        (
            tokio::spawn(async move { self.dispatcher.dispatch().await }),
            shutdown_token,
        )
    }
}

pub async fn handle_command(
    message: Message,
    tg: Arc<Bot>,
    command: Command,
    config: Arc<Config>,
) -> Result<()> {
    async fn handle(
        message: &Message,
        tg: &Bot,
        command: Command,
        config: &Arc<Config>,
    ) -> Result<()> {
        match command {
            Command::Help => {
                let _ = tg
                    .send_message(message.chat.id, Command::descriptions().to_string())
                    .await;
            }
            Command::Sub(args) => {
                let chat_id = message.chat.id;

                let error = check_args(args.clone());

                if !error.is_empty() {
                    tg.send_message(
                        chat_id,
                        "Please provide the arguments needed. Check /help for guidance.",
                    )
                    .await?;
                    return Ok(());
                }

                let SubscriptionArgs {
                    location,
                    target_yield,
                    min_size,
                    max_size,
                } = args;

                let message_target_yield = match target_yield {
                    Some(value) => f64::from(value),
                    None => {
                        tg.send_message(
                            chat_id,
                            "Target yield is missing. Please provide it, e.g. yield=10.",
                        )
                        .await?;
                        return Ok(());
                    }
                };

                let size_range = match (min_size, max_size) {
                    (Some(min), Some(max)) => (f64::from(min), f64::from(max)),
                    _ => {
                        tg.send_message(
                            chat_id,
                            "Both min_size and max_size must be provided, e.g. min_size=50 max_size=60.",
                        )
                        .await?;
                        return Ok(());
                    }
                };

                match subscribe_to_watchlist(
                    size_range,
                    message_target_yield,
                    location,
                    chat_id,
                    tg,
                    config.clone(),
                )
                .await
                {
                    Ok(()) => {}
                    Err(e) => {
                        tg.send_message(
                            chat_id,
                            format!("Could not subscribe. Please try again: {}", e),
                        )
                        .await?;
                    }
                }
            }
            Command::Unsub(watchlist_id) => {
                let chat_id = message.chat.id.0;

                let Some(watchlist_id) = watchlist_id else {
                    tg.send_message(
                        message.chat.id,
                        "Please provide the watchlist ID, e.g. /unsub 42.",
                    )
                    .await?;
                    return Ok(());
                };

                // Check if watchlist for this place already exists for this chat
                let existing: Vec<Watchlist> = db::watchlist::get_for_chat(config, chat_id)
                    .iter()
                    .filter(|watchlist| watchlist.id == watchlist_id)
                    .map(|item| item.to_owned())
                    .collect();

                if existing.is_empty() {
                    tg.send_message(message.chat.id, "You don't have a watchlist with this ID")
                        .await?;
                } else {
                    db::watchlist::delete(config, watchlist_id);
                    tg.send_message(message.chat.id, "Deleted watchlist!")
                        .await?;
                }
            }
            Command::ListWatchlists => {
                let chat_id = message.chat.id.0;

                // Check if watchlist for this place already exists for this chat
                let existing: Vec<Watchlist> = db::watchlist::get_for_chat(config, chat_id);
                let formatted: Vec<String> = existing
                    .iter()
                    .enumerate()
                    .map(|(index, watchlist)| {
                        format!(
                            "{}: \n Id: {} Location: {} Target Yield: {} Size: {}:{} \n\n",
                            index + 1,
                            watchlist.id.clone(),
                            watchlist.location_name.clone(),
                            watchlist.target_yield.unwrap(),
                            watchlist.target_size_min.unwrap(),
                            watchlist.target_size_max.unwrap()
                        )
                    })
                    .collect();

                if formatted.is_empty() {
                    tg.send_message(message.chat.id, "No subs").await?;
                } else {
                    let joined_formatted = formatted.join("\n");
                    tg.send_message(message.chat.id, joined_formatted).await?;
                }
            }
            Command::GetAll(watchlist_id) => {
                let Some(watchlist_id) = watchlist_id else {
                    tg.send_message(
                        message.chat.id,
                        "Please provide the watchlist ID, e.g. /getall 42.",
                    )
                    .await?;
                    return Ok(());
                };

                let chat_id = message.chat.id.0;

                let all_apartments_result =
                    db::apartment::get_all_for_watchlist(config, chat_id, watchlist_id);
                let mut all_apartments: Option<Vec<Apartment>> = None;

                match all_apartments_result {
                    Ok(aps) => all_apartments = Some(aps),
                    Err(e) => {
                        tg.send_message(message.chat.id, format!("Error while fetching: {}", e))
                            .await?;
                    }
                };

                if let Some(aps) = all_apartments {
                    send_formatted_message_all(tg, message, aps).await?;
                }
            }
            Command::GetMatching(watchlist_id) => {
                let Some(watchlist_id) = watchlist_id else {
                    tg.send_message(
                        message.chat.id,
                        "Please provide the watchlist ID, e.g. /getmatching 42.",
                    )
                    .await?;
                    return Ok(());
                };

                let chat_id = message.chat.id.0;
                let apartments_result =
                    db::apartment::get_matching_for_watchlist(config, chat_id, watchlist_id);
                let mut apartments: Option<Vec<Apartment>> = None;

                match apartments_result {
                    Ok(aps) => apartments = Some(aps),
                    Err(e) => {
                        tg.send_message(message.chat.id, format!("Error while fetching: {}", e))
                            .await?;
                    }
                };

                if let Some(aps) = apartments {
                    send_formatted_message_all_valid(tg, message, aps, watchlist_id).await?;
                }
            }
        };
        Ok(())
    }

    if let Err(err) = handle(&message, &tg, command, &config).await {
        error!("Failed to handle message: {}", err);
        tg.send_message(message.chat.id, "Something went wrong, please try again")
            .await?;
    }

    Ok(())
}

fn parse_subscribe_message(input: String) -> Result<(SubscriptionArgs,), ParseError> {
    lazy_static! {
        static ref LOCATION_STRING_REGEX: Regex = Regex::new(r"^[^\s]+").unwrap();
        static ref MIN_SIZE_REGEX: Regex = Regex::new(r"\bmin_size=(\d+)\b").unwrap();
        static ref MAX_SIZE_REGEX: Regex = Regex::new(r"\bmax_size=(\d+)\b").unwrap();
        static ref YIELD_REGEX: Regex = Regex::new(r"\byield=(\d+)\b").unwrap();
    }

    let location = LOCATION_STRING_REGEX
        .find(&input)
        .ok_or_else(|| ParseError::Custom("No location given".into()))?
        .as_str()
        .to_string();

    let min_size: Option<u32> = MIN_SIZE_REGEX
        .captures(&input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    let max_size: Option<u32> = MAX_SIZE_REGEX
        .captures(&input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    let target_yield: Option<u32> = YIELD_REGEX
        .captures(&input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    let args = SubscriptionArgs {
        location,
        target_yield,
        min_size,
        max_size,
    };

    Ok((args,))
}

fn parse_string_to_int_message(input: String) -> Result<(Option<i32>,), ParseError> {
    if input.trim().is_empty() {
        return Ok((None,));
    }

    match input.trim().parse::<i32>() {
        Ok(id) => Ok((Some(id),)),
        Err(_) => Err(ParseError::Custom(
            "Unable to parse the supplied ID.".into(),
        )),
    }
}

async fn send_formatted_message_all_valid(
    tg: &Bot,
    message: &Message,
    apartments: Vec<Apartment>,
    watchlist_id: i32,
) -> Result<()> {
    let formatted: Vec<String> = apartments
        .iter()
        .enumerate()
        .map(|(index, apartment)| {
            format!(
                "{}: \n Location: {} \n Size: {:.1} m^2 \n Price: {} EUR \n Estimated Rent: {} EUR \n Estimated Yield: {:.2}% \n Url: {}",
                index,
                apartment
                    .location_name
                    .as_ref()
                    .unwrap_or(&"N/A".to_string()),
                apartment.size.unwrap_or(0.0),
                apartment.price.unwrap_or(0),
                apartment.rent.unwrap_or_default(),
                apartment.estimated_yield.unwrap_or(0.0),
                apartment.url.as_ref().unwrap_or(&"N/A".to_string())
            )
        })
        .collect();
    tg.send_message(
        message.chat.id,
        format!(
            "The following apartments are over the target yield for watchlist {}",
            watchlist_id
        ),
    )
    .await?;
    for message_to_send in formatted {
        tg.send_message(message.chat.id, message_to_send).await?;
    }
    Ok(())
}

async fn send_formatted_message_all(
    tg: &Bot,
    message: &Message,
    apartments: Vec<Apartment>,
) -> Result<()> {
    let formatted: Vec<String> = apartments
        .iter()
        .enumerate()
        .map(|(index, apartment)| {
            format!(
                "{}: \n Location: {} \n Size: {:.1} m^2 \n Price: {} EUR \n Estimated Yield: {:.2}%",
                index,
                apartment
                    .location_name
                    .as_ref()
                    .unwrap_or(&"N/A".to_string()),
                apartment.size.unwrap_or(0.0),
                apartment.price.unwrap_or(0),
                apartment.estimated_yield.unwrap_or(0.0)
            )
        })
        .collect();
    for message_to_send in formatted {
        tg.send_message(message.chat.id, message_to_send).await?;
    }
    Ok(())
}

pub fn format_apartment_message(watchlist: &Watchlist, apartment: &Apartment) -> String {
    format!(
        "Found a new apartment matching your criteria for watchlist {} \n\n Location: {} \n Size: {:.1} m^2 \n Price: {} EUR \n Estimated Rent: {} EUR \n Estimated Yield: {:.2}% \n Url: {}",
        watchlist.id,
        apartment
            .location_name
            .as_ref()
            .unwrap_or(&"N/A".to_string()),
        apartment.size.unwrap_or(0.0),
        apartment.price.unwrap_or(0),
        apartment.rent.unwrap_or_default(),
        apartment.estimated_yield.unwrap_or(0.0),
        apartment.url.as_ref().unwrap_or(&"N/A".to_string())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subscribe_message_only_watchlist() {
        let args = parse_subscribe_message("testlocation".to_string()).unwrap();
        assert_eq!(
            args.0,
            SubscriptionArgs {
                location: "testlocation".to_string(),
                target_yield: None,
                min_size: None,
                max_size: None
            },
        )
    }

    #[test]
    fn test_parse_subscribe_message_without_declarations() {
        let args = parse_subscribe_message("testlocation 60 10".to_string()).unwrap();
        assert_eq!(
            args.0,
            SubscriptionArgs {
                location: "testlocation".to_string(),
                target_yield: None,
                min_size: None,
                max_size: None
            },
        );
    }

    #[test]
    fn test_parse_subscribe_message_correct_format() {
        let args =
            parse_subscribe_message("testlocation yield=10 min_size=50 max_size=65".to_string())
                .unwrap();
        assert_eq!(
            args.0,
            SubscriptionArgs {
                location: "testlocation".to_string(),
                target_yield: Some(10),
                min_size: Some(50),
                max_size: Some(65)
            },
        )
    }
}
