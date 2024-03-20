use crate::{bot::bot_types, db};
use anyhow::Result;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use std::{env, sync::Arc};
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

// Inspiration: https://github.com/raine/tgreddit

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Help")]
    Help,
    #[command(
        description = "Subscribe to a location watchlist",
        parse_with = parse_subscribe_message
    )]
    Sub(SubscriptionArgs),
    #[command(description = "Unsubscribe to a watchlist. Use watchlist ID.")]
    Unsub(String),
    #[command(description = "List current active watchlist subscriptions")]
    ListSubs,
    #[command(description = "Get all apartments/houses in watchlist")]
    GetAll(String),
    #[command(
        description = "Get all apartments/houses in watchlist that are above or equal to the yield goal"
    )]
    GetAllValid(String),
}

pub struct ApatoBot {
    pub dispatcher: Dispatcher<Arc<Bot>, anyhow::Error, DefaultKey>,
    pub tg: Arc<Bot>,
}

impl ApatoBot {
    pub async fn new() -> Result<Self> {
        dotenv().ok();

        let telegram_bot_token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set");

        let tg = Arc::new(Bot::new(telegram_bot_token));
        tg.set_my_commands(Command::bot_commands()).await?;

        let handler = Update::filter_message().branch(
            // TODO Fix this...
            dptree::filter(|msg: Message| msg.from().map(|user| true).unwrap_or_default())
                .filter_command::<Command>()
                .endpoint(handle_command),
        );

        let dispatcher = Dispatcher::builder(tg.clone(), handler)
            .default_handler(|upd| async move { println!("{:?}", upd) })
            .error_handler(LoggingErrorHandler::with_custom_text(
                "an error has occurred in the dispatcher",
            ))
            .build();

        let bot = ApatoBot {
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

pub async fn handle_command(message: Message, tg: Arc<Bot>, command: Command) -> Result<()> {
    async fn handle(message: &Message, tg: &Bot, command: Command) -> Result<()> {
        match command {
            Command::Help => {
                let _ = tg
                    .send_message(message.chat.id, Command::descriptions().to_string())
                    .await;
            }
            Command::Sub(mut args) => {
                let db = db::establish_connection();
                let chat_id = message.chat.id.0;
            }
            Command::Unsub(_) => todo!(),
            Command::ListSubs => todo!(),
            Command::GetAll(_) => todo!(),
            Command::GetAllValid(_) => todo!(),
        };

        Ok(())
    }

    if let Err(err) = handle(&message, &tg, command).await {
        error!("failed to handle message: {}", err);
        tg.send_message(message.chat.id, "Something went wrong")
            .await?;
    }

    Ok(())
}

fn parse_subscribe_message(input: String) -> Result<(SubscriptionArgs,), ParseError> {
    lazy_static! {
        static ref LOCATION_STRING_REGEX: Regex = Regex::new(r"^[^\s]+").unwrap();
        static ref YIELD_REGEX: Regex = Regex::new(r"\byield=(\d+)\b").unwrap();
    }

    let location = LOCATION_STRING_REGEX
        .find(&input)
        .ok_or_else(|| ParseError::Custom("No location given".into()))?
        .as_str()
        .to_string();

    let yield_goal: Option<u32> = YIELD_REGEX
        .captures(&input)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    let args = SubscriptionArgs {
        location,
        size: Some(1),
        yield_goal,
    };

    Ok((args,))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_parse_subscribe_message_only_subreddit() {
//         let args = parse_subscribe_message("AnimalsBeingJerks".to_string()).unwrap();
//         assert_eq!(
//             args.0,
//             SubscriptionArgs {
//                 subreddit: "AnimalsBeingJerks".to_string(),
//                 limit: None,
//                 time: None,
//                 filter: None,
//             },
//         )
//     }

//     #[test]
//     fn test_parse_subscribe_message_strips_prefix() {
//         let args = parse_subscribe_message("r/AnimalsBeingJerks".to_string()).unwrap();
//         assert_eq!(
//             args.0,
//             SubscriptionArgs {
//                 subreddit: "AnimalsBeingJerks".to_string(),
//                 limit: None,
//                 time: None,
//                 filter: None,
//             },
//         );

//         let args = parse_subscribe_message("/r/AnimalsBeingJerks".to_string()).unwrap();
//         assert_eq!(
//             args.0,
//             SubscriptionArgs {
//                 subreddit: "AnimalsBeingJerks".to_string(),
//                 limit: None,
//                 time: None,
//                 filter: None,
//             },
//         )
//     }

//     #[test]
//     fn test_parse_subscribe_message() {
//         let args =
//             parse_subscribe_message("AnimalsBeingJerks limit=5 time=week filter=video".to_string())
//                 .unwrap();
//         assert_eq!(
//             args.0,
//             SubscriptionArgs {
//                 subreddit: "AnimalsBeingJerks".to_string(),
//                 limit: Some(5),
//                 time: Some(TopPostsTimePeriod::Week),
//                 filter: Some(PostType::Video),
//             },
//         )
//     }
// }
