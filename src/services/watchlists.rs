use std::sync::Arc;

use anyhow::{anyhow, Result};

use crate::{
    config::Config,
    db,
    models::{
        apartment::Apartment,
        watchlist::{SizeTarget, Watchlist},
    },
    oikotie::oikotie::{Location, Oikotie},
};

pub async fn subscribe(
    config: Arc<Config>,
    chat_id: i64,
    location_query: String,
    size: (f64, f64),
    target_yield: f64,
) -> Result<Watchlist> {
    let existing = db::watchlist::get_for_chat_and_location(&config, chat_id, &location_query);
    if let Some(current) = existing.first() {
        db::watchlist::update_yield(&config, current.id, target_yield).await?;
        let mut updated = current.clone();
        updated.target_yield = Some(target_yield);
        return Ok(updated);
    }

    let mut oikotie_client = Oikotie::new().await;
    let locations = oikotie_client
        .get_locations_for_zip_code(&location_query)
        .await?;

    if locations.is_empty() {
        return Err(anyhow!("Did not find any location with that query"));
    }

    let location_card = &locations[0].card;
    let watchlist_location = Location {
        id: location_card.card_id as i32,
        level: location_card.card_type as i32,
        name: location_card.name.clone(),
    };
    let location_handle = watchlist_location.clone();

    let mut target_size = SizeTarget::empty();
    target_size.min = Some(size.0 as i32);
    target_size.max = Some(size.1 as i32);

    db::watchlist::insert(
        &config,
        watchlist_location,
        chat_id,
        Some(target_yield),
        target_size,
    );

    let created = db::watchlist::get_for_chat(&config, chat_id)
        .into_iter()
        .find(|w| w.location_id == location_handle.id && w.location_level == location_handle.level)
        .ok_or_else(|| anyhow!("Failed to create watchlist"))?;

    Ok(created)
}

pub fn list(config: &Arc<Config>, chat_id: i64) -> Vec<Watchlist> {
    db::watchlist::get_for_chat(config, chat_id)
}

pub fn delete(config: &Arc<Config>, chat_id: i64, watchlist_id: i32) -> Result<()> {
    let existing = db::watchlist::get_for_chat(config, chat_id);
    if existing.iter().any(|w| w.id == watchlist_id) {
        db::watchlist::delete(config, watchlist_id);
        Ok(())
    } else {
        Err(anyhow!("You don't have a watchlist with this ID"))
    }
}

pub fn get_all_apartments(
    config: &Arc<Config>,
    chat_id: i64,
    watchlist_id: i32,
) -> Result<Vec<Apartment>> {
    db::apartment::get_all_for_watchlist(config, chat_id, watchlist_id)
}

pub fn get_matching_apartments(
    config: &Arc<Config>,
    chat_id: i64,
    watchlist_id: i32,
) -> Result<Vec<Apartment>> {
    db::apartment::get_matching_for_watchlist(config, chat_id, watchlist_id)
}
