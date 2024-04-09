use crate::{
    config::Config,
    db,
    models::{apartment::Apartment, watchlist::Watchlist},
    oikotie::oikotie::{Location, Oikotie},
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
        description = "Subscribe to a location watchlist. Provide the args in the following format: < /sub {location name} size={size (m^2)} yield={target yield}. > \n\n Example: \n '< /sub ullanlinna size=60 yield=10 >",
        parse_with = parse_subscribe_message
    )]
    Sub(SubscriptionArgs),

    #[command(
        description = "Unsubscribe to a watchlist. Use watchlist ID.",
        parse_with = parse_string_to_int_message
    )]
    Unsub(i32),

    #[command(description = "List current active watchlist subscriptions")]
    ListWatchlists,

    #[command(description = "Get all apartments/houses in watchlist",
        parse_with = parse_string_to_int_message
    )]
    GetAll(i32),

    #[command(
        description = "Get all apartments/houses in watchlist that are above or equal to the yield goal",
        parse_with = parse_string_to_int_message
    )]
    GetAllValid(i32),
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
                let chat_id = message.chat.id.0;

                if args.location.is_empty() && args.size.is_none() && args.yield_goal.is_none() {
                    tg.send_message(
                        message.chat.id,
                        "Please provide the arguments needed. Check /help for guidance.",
                    )
                    .await?;
                }

                // Check if watchlist for this place already exists for this chat
                let existing =
                    db::watchlist::get_for_chat_and_location(config, chat_id, &args.location);

                if !existing.is_empty() {
                    tg.send_message(
                        message.chat.id,
                        "You already have a watchlist for this location. Updating goal yield...",
                    )
                    .await?;

                    match db::watchlist::update_yield(
                        config,
                        existing[0].id,
                        args.yield_goal.unwrap().into(),
                    )
                    .await
                    {
                        Ok(()) => (), // TODO Clean up this :D
                        Err(e) => {
                            tg.send_message(
                                message.chat.id,
                                format!("Error while updating yield: {}", e),
                            )
                            .await?;
                            
                        }
                    };

                    return Ok(());
                }

                // Create new watchlist
                let mut oikotie_client: Oikotie = Oikotie::new().await;
                let location_id_response = oikotie_client.get_location_id(&args.location).await;
                let mut location: Option<Location> = None;

                match location_id_response {
                    Ok(location_id) => {
                        location = Some(Location {
                            id: location_id as i32,
                            level: 4, // TODO maybe not just hardcode this
                            name: args.location,
                        })
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        tg.send_message(message.chat.id, err_str).await?;
                    }
                }

                if let Some(loc) = location {
                    db::watchlist::insert(
                        config,
                        loc,
                        chat_id,
                        Some(args.yield_goal.unwrap_or(0) as f64),
                    );
                    tg.send_message(message.chat.id, "Added to your watchlist!")
                        .await?;
                }
            }
            Command::Unsub(watchlist_id) => {
                let chat_id = message.chat.id.0;

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
                            "{}: Id: {};    Location: {};   Target Yield: {}",
                            index + 1,
                            watchlist.id.clone(),
                            watchlist.location_name.clone(),
                            watchlist.goal_yield.unwrap()
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
                let chat_id = message.chat.id.0;

                let all_apartments_result =
                    db::apartment::get_all_for_watchlist(config, chat_id, watchlist_id);
                let mut all_apartments: Option<Vec<Apartment>> = None;

                match all_apartments_result {
                    Ok(aps) => all_apartments = Some(aps),
                    Err(e) => {
                        tg.send_message(
                            message.chat.id,
                            format!("Error while fetching: {}", e),
                        )
                        .await?;
                    }
                };

                if let Some(aps) = all_apartments {
                    send_formatted_message_all(tg, message, aps).await?;
                }
            }
            Command::GetAllValid(watchlist_id) => {
                let chat_id = message.chat.id.0;
                let apartments_result =
                    db::apartment::get_all_valid_for_watchlist(config, chat_id, watchlist_id);
                let mut apartments: Option<Vec<Apartment>> = None;

                match apartments_result {
                    Ok(aps) => apartments = Some(aps),
                    Err(e) => {
                        tg.send_message(
                            message.chat.id,
                            format!("Error while fetching: {}", e),
                        )
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
        static ref SIZE_REGEX: Regex = Regex::new(r"\bsize=(\d+)\b").unwrap();
        static ref YIELD_REGEX: Regex = Regex::new(r"\byield=(\d+)\b").unwrap();
    }

    let location = LOCATION_STRING_REGEX
        .find(&input)
        .ok_or_else(|| ParseError::Custom("No location given".into()))?
        .as_str()
        .to_string();

    let size: Option<u32> = SIZE_REGEX
        .captures(&input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    let yield_goal: Option<u32> = YIELD_REGEX
        .captures(&input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    let args = SubscriptionArgs {
        location,
        size,
        yield_goal,
    };

    Ok((args,))
}

fn parse_string_to_int_message(input: String) -> Result<(i32,), ParseError> {
    let input = input.parse::<i32>();
    match input {
        Ok(id) => Ok((id,)),
        Err(e) => Err(ParseError::Custom(Box::from(e))),
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
                "{}: \n Location: {} \n Size: {} \n Price: {} \n Estimated Rent: {} \n Estimated Yield: {} \n Url: {}",
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
                "{}: \n Location: {} \n Size: {} \n Price: {} \n Estimated Yield: {}",
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

pub fn format_apartment_message(apartment: &Apartment) -> String {
    format!(
        "Location: {} \n Size: {} \n Price: {} \n Estimated Rent: {} \n Estimated Yield: {} \n Url: {}",
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
                size: None,
                yield_goal: None
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
                size: None,
                yield_goal: None
            },
        );
    }

    #[test]
    fn test_parse_subscribe_message_correct_format() {
        let args = parse_subscribe_message("testlocation yield=10 size=50".to_string()).unwrap();
        assert_eq!(
            args.0,
            SubscriptionArgs {
                location: "testlocation".to_string(),
                size: Some(50),
                yield_goal: Some(10)
            },
        )
    }
}
