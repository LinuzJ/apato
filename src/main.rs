use anyhow::Result;
use apato::{self, bot::bot::ApatoBot, logger::setup_logger};
use log::error;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    let _ = setup_logger();
    // Launch consumer process
    let consumer_handle = apato::spawn_apato().await;

    // Telegram bot
    let bot = ApatoBot::new().await?;
    let (bot_handle, s) = bot.spawn();

    if let Err(err) = tokio::try_join!(consumer_handle, bot_handle) {
        error!("Error: {:?}", err)
    }

    Ok(())
}
