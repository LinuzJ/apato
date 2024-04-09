use std::ops::Sub;
use std::sync::Arc;

use crate::config::Config;
use crate::db::apartment::get_apartments_within_period;
use crate::models::apartment::InsertableApartment;
use crate::models::watchlist::Watchlist;
use crate::oikotie::helpers;
use crate::oikotie::tokens;

use anyhow::anyhow;
use anyhow::{Error, Result};
use chrono::Duration;
use chrono::Utc;
use helpers::create_location_string;
use log::error;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_this_or_that::as_u64;
use tokens::{get_tokens, OikotieTokens};

use super::helpers::estimated_rent;
use super::helpers::get_rent_regex;

#[derive(Debug, Clone)]
pub struct Location {
    pub id: i32,
    pub level: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocationApiCard {
    name: String,
    card_id: u32,
    card_type: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocationApiResponseItem {
    card: LocationApiCard,
}

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    id: u32,
    url: String,
    description: String,
    rooms: u32,
    #[serde(deserialize_with = "price_int_or_string")]
    price: String,
    published: String,
    size: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Price {
    #[serde(deserialize_with = "as_u64")]
    price: u64,
}

impl Price {
    fn empty() -> Price {
        Price { price: 0 }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdData {
    #[serde(deserialize_with = "as_u64")]
    maintenance_fee: u64,
    #[serde(deserialize_with = "as_u64")]
    size: u64,
    room_configuration: String,
}

impl AdData {
    fn empty() -> AdData {
        AdData {
            maintenance_fee: 0,
            size: 0,
            room_configuration: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OitkotieCardsApiResponse {
    found: u32,
    cards: Vec<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OitkotieCardApiResponse {
    card_id: i32,
    ad_data: AdData,
    price_data: Price,
    status: i32,
}

pub struct RentalData {
    pub rent: i32,
    pub size: f32,
}

// Custom deserialization for price field as it can be int or String
fn price_int_or_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => {
            (num.as_f64().ok_or(de::Error::custom("Invalid number"))? as i32).to_string()
        }
        _ => return Err(de::Error::custom("wrong type")),
    })
}

impl OitkotieCardApiResponse {
    fn empty() -> OitkotieCardApiResponse {
        OitkotieCardApiResponse {
            card_id: 0,
            ad_data: AdData::empty(),
            price_data: Price::empty(),
            status: 0,
        }
    }
}

#[derive(Debug)]
pub struct Oikotie {
    pub tokens: Option<Box<OikotieTokens>>,
}

impl Oikotie {
    pub async fn new() -> Oikotie {
        Oikotie {
            tokens: get_tokens().await,
        }
    }

    /*
       Use Oikotie's search API to find location ID based on text query
    */
    pub async fn get_location_id(&mut self, location_string: &str) -> Result<u32> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let response: Result<Vec<LocationApiResponseItem>, reqwest::Error> =
            fetch_location_id(&self.tokens.as_ref().unwrap(), location_string).await;

        let potential_locations = match response {
            Ok(l) => Some(l),
            Err(e) => {
                error!("Error while fetching location id from Oikotie: {}", e);
                return Err(e.into());
            }
        };

        if let Some(locations) = potential_locations {
            if locations.len() == 0 {
                return Err(anyhow!(
                    "Did not find any valid location for '{}', please try again!",
                    location_string
                ));
            }

            return Ok(locations[0].card.card_id);
        } else {
            return Err(anyhow!(
                "Did not find any valid location for '{}', please try again!",
                location_string
            ));
        }
    }

    /*
       Fecthes all apartments for a certain location
    */
    pub async fn get_apartments(
        &mut self,
        config: Arc<Config>,
        watchlist: &Watchlist,
    ) -> Option<Vec<InsertableApartment>> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let is_handling_rent = false;

        let location: &Location = &Location {
            id: watchlist.id,
            level: watchlist.location_level,
            name: watchlist.location_name.clone(),
        };

        let cards_response: Result<OitkotieCardsApiResponse, reqwest::Error> =
            fetch_apartments(&self.tokens.as_ref().unwrap(), location.clone(), false).await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(_e) => return None,
        };

        let mut cards_iter: std::slice::Iter<'_, Card> = cards.iter();
        let mut apartments: Vec<InsertableApartment> = Vec::new();

        while let Some(card) = cards_iter.next() {
            let has_been_updated_recently = has_been_updated_recently(config.clone(), card);

            if !has_been_updated_recently {
                let apartment: InsertableApartment = card_into_complete_apartment(
                    &self.tokens.as_ref().unwrap(),
                    card,
                    location,
                    Some(watchlist.id),
                    is_handling_rent,
                )
                .await;
                apartments.push(apartment);
            }
        }

        return Some(apartments);
    }

    /*
       Fecthes all rental apartments for a certain location
    */
    pub async fn get_rental_data(&mut self, location: &Location) -> Result<Vec<RentalData>> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let is_handling_rent = true;

        let location: Location = Location {
            id: location.id,
            level: location.level,
            name: location.name.clone(),
        };

        let cards_response: Result<OitkotieCardsApiResponse, reqwest::Error> = fetch_apartments(
            &self.tokens.as_ref().unwrap(),
            location.clone(),
            is_handling_rent,
        )
        .await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(_e) => return Err(anyhow!("Error while fetching cards")),
        };

        let mut cards_iter: std::slice::Iter<'_, Card> = cards.iter();
        let mut data: Vec<RentalData> = Vec::new();

        while let Some(card) = cards_iter.next() {
            let rent = get_rent_regex(card.price.clone());
            let rent_data = RentalData {
                rent,
                size: card.size,
            };
            data.push(rent_data);
        }

        return Ok(data);
    }

    /*
       Calculates and returns the estimated rent for a given location
    */
    pub async fn get_estimated_rent(
        &mut self,
        apartment: &InsertableApartment,
    ) -> Result<i32, Error> {
        let location = &Location {
            id: apartment.location_id.unwrap(),
            level: apartment.location_level.unwrap(),
            name: apartment.location_name.clone().unwrap(),
        };
        let rental_apartments_nearby = self.get_rental_data(location).await;
        let size = apartment.size.unwrap_or_default();

        match rental_apartments_nearby {
            Ok(rental_data) => Ok(estimated_rent(size as f32, rental_data)),
            Err(e) => Err(anyhow!(
                "PRODUCER ERROR while calculating rent: {}",
                e.to_string()
            )),
        }
    }
}

async fn fetch_location_id(
    tokens: &OikotieTokens,
    location_string: &str,
) -> Result<Vec<LocationApiResponseItem>, reqwest::Error> {
    let client: reqwest::Client = reqwest::Client::new();

    // Create request with needed token headers
    let oikotie_cards_api_url = String::from("https://asunnot.oikotie.fi/api/5.0/location/");
    let params: Vec<(&str, &str)> = vec![("query", location_string), ("card_type", "4")];

    let mut headers: HeaderMap = HeaderMap::new();
    match HeaderValue::from_str(&tokens.loaded) {
        Ok(loaded) => headers.insert("ota-loaded", loaded),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(&tokens.cuid) {
        Ok(cuid) => headers.insert("ota-cuid", cuid),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(&tokens.token) {
        Ok(token) => headers.insert("ota-token", token),
        Err(_e) => todo!(),
    };

    // Perform the actual request
    let response = client
        .get(oikotie_cards_api_url)
        .query(&params)
        .headers(headers)
        .send()
        .await;

    let api_response: Vec<LocationApiResponseItem> = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    return Ok(api_response);
}

async fn fetch_card(
    tokens: &OikotieTokens,
    card_id: String,
) -> Result<OitkotieCardApiResponse, reqwest::Error> {
    let client: reqwest::Client = reqwest::Client::new();

    // Create request with needed token headers
    let mut oikotie_cards_api_url = String::from("https://asunnot.oikotie.fi/api/5.0/card/");
    oikotie_cards_api_url.push_str(&card_id.to_owned());

    let mut headers: HeaderMap = HeaderMap::new();
    match HeaderValue::from_str(&tokens.loaded) {
        Ok(loaded) => headers.insert("ota-loaded", loaded),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(&tokens.cuid) {
        Ok(cuid) => headers.insert("ota-cuid", cuid),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(&tokens.token) {
        Ok(token) => headers.insert("ota-token", token),
        Err(_e) => todo!(),
    };

    // Perform the actual request
    let response = client
        .get(oikotie_cards_api_url)
        .headers(headers)
        .send()
        .await;

    let api_response: OitkotieCardApiResponse = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    return Ok(api_response);
}

async fn fetch_apartments(
    tokens: &OikotieTokens,
    location: Location,
    get_rentals: bool,
) -> Result<OitkotieCardsApiResponse, reqwest::Error> {
    let oikotie_cards_api_url = "https://asunnot.oikotie.fi/api/cards";

    let client: reqwest::Client = reqwest::Client::new();
    let locations: String = create_location_string(location.id, location.level, location.name);
    let params: Vec<(&str, &str)> = vec![
        ("cardType", if get_rentals { "101" } else { "100" }),
        ("locations", &locations),
    ];
    let mut headers: HeaderMap = HeaderMap::new();

    match HeaderValue::from_str(&tokens.loaded) {
        Ok(loaded) => headers.insert("ota-loaded", loaded),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(&tokens.cuid) {
        Ok(cuid) => headers.insert("ota-cuid", cuid),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(&tokens.token) {
        Ok(token) => headers.insert("ota-token", token),
        Err(_e) => todo!(),
    };

    let response = client
        .get(oikotie_cards_api_url)
        .query(&params)
        .headers(headers)
        .send()
        .await;

    let api_response: OitkotieCardsApiResponse = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    return Ok(api_response);
}

async fn card_into_complete_apartment(
    tokens: &OikotieTokens,
    card: &Card,
    location: &Location,
    optional_watchlist_id: Option<i32>,
    is_handling_rent: bool,
) -> InsertableApartment {
    // TODO FIX THIS TO HANDLE 5.0 API
    // Fetch card data that includes total price information
    let card_data: OitkotieCardApiResponse = match fetch_card(tokens, card.id.to_string()).await {
        Ok(c) => c,
        Err(_e) => {
            error!(
                "Did not fetch card data for card {:?}. Error is {:?}",
                card.id, _e
            );
            OitkotieCardApiResponse::empty()
        }
    };

    let watchlist_id = match optional_watchlist_id {
        Some(id) => id,
        None => -1,
    };

    InsertableApartment {
        card_id: Some(card.id.to_string()),
        location_id: Some(location.id),
        location_level: Some(location.level),
        location_name: Some(location.name.clone()),
        size: Some(card.size as f64),
        rooms: Some(card.rooms as i32),
        price: Some(card_data.price_data.price as i32),
        additional_costs: Some(card_data.ad_data.maintenance_fee as i32),
        rent: Some(0),
        estimated_yield: Some(0.0),
        url: Some(card.url.clone()),
        watchlist_id: watchlist_id,
    }
}

fn has_been_updated_recently(config: Arc<Config>, card: &Card) -> bool {
    let cooldown_seconds = 2 * 24 * 60 * 60; // Two days
    let now = Utc::now().naive_local();
    let cooldown_period = now.sub(Duration::seconds(cooldown_seconds));

    let apartments =
        match get_apartments_within_period(&config, card.id.to_string(), cooldown_period) {
            Ok(aps) => aps,
            Err(e) => {
                error!("Error while querying apartmetns withing period");
                vec![]
            }
        };

    return apartments.len() != 0;
}
