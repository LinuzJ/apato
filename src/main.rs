use std::sync::Arc;

use anyhow::Result;
use apato::{
    self, bot::bot::ApatoBot, config, logger::setup_logger,
    producer::pricing_producer::PricingProducer,
};
use config::Config;
use log::error;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    setup_logger()?;

    let config: Arc<Config> = Arc::new(config::read_config());

    // Telegram bot
    let bot = ApatoBot::new(&config).await?;
    let (bot_handle, _s) = bot.spawn();

    // Launch producer process
    let producer_handle =
        tokio::task::spawn(async move { PricingProducer::run(config.clone()).await });

    if let Err(err) = tokio::try_join!(producer_handle, bot_handle) {
        error!("Error: {:?}", err)
    }

    Ok(())
}
