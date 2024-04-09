extern crate chrono;
extern crate diesel;
extern crate tokio;
pub mod bot;
pub mod config;
mod db;
mod interest_rate;
pub mod logger;
pub mod models;
mod oikotie;
pub mod producer;

use bot::bot::ApatoTelegramBot;
use logger::setup_logger;
pub use producer::calculate_irr;
use producer::pricing_producer::PricingProducer;

use std::sync::Arc;

use anyhow::Result;
use config::Config;
use log::error;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    setup_logger()?;

    let config: Arc<Config> = Arc::new(config::read_config());

    // Telegram bot
    let bot = ApatoTelegramBot::new(config.clone()).await?;
    let (bot_handle, _s) = bot.spawn();

    // Launch producer process
    let producer_handle =
        tokio::task::spawn(async move { PricingProducer::run(config.clone()).await });

    if let Err(err) = tokio::try_join!(producer_handle, bot_handle) {
        error!("Error: {:?}", err)
    }

    Ok(())
}
