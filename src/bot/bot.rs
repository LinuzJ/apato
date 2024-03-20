use crate::bot::bot_types;
use anyhow::Result;
use dotenvy::dotenv;
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
    pub async fn new() -> Result<ApatoBot> {
        dotenv().ok();

        let telegram_bot_token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set");

        let tg = Arc::new(Bot::new(telegram_bot_token));
        tg.set_my_commands(Command::bot_commands()).await;

        let handler = Update::filter_message().branch(
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
            Command::Sub(_) => todo!(),
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
            .await;
    }

    Ok(())
}

fn parse_subscribe_message(input: String) -> Result<(SubscriptionArgs,), ParseError> {
    let args = SubscriptionArgs {
        location: 1.to_string(),
        size: Some(1),
        yield_goal: 1,
    };

    Ok((args,))
}
