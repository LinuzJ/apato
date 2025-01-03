use std::sync::Arc;

use crate::config::Config;
use crate::db;
use crate::db::apartment_watchlist::get_watchlist_apartment_connector;
use crate::models::apartment::InsertableApartment;
use crate::models::watchlist::SizeTarget;
use crate::models::watchlist::Watchlist;
use crate::oikotie::helpers;
use crate::oikotie::tokens;
use crate::send_request;
use crate::RequestType;
use crate::URLS;

use anyhow::anyhow;
use anyhow::{Error, Result};
use helpers::create_location_string;
use log::error;

use reqwest::header::{HeaderMap, HeaderValue};
use serde::de;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_this_or_that::as_u64;
use tokens::{get_tokens, OikotieTokens};

use super::helpers::estimate_rent;
use super::helpers::get_rent_regex;
use super::oikotie_types::CardTypes;

#[derive(Debug, Clone)]
pub struct Location {
    pub id: i32,
    pub level: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocationCard {
    pub name: String,
    pub card_id: u32,
    pub card_type: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationResponse {
    pub card: LocationCard,
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
struct CardsResponse {
    found: u32,
    cards: Vec<Card>,
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
#[serde(rename_all = "camelCase")]
struct CardResponse {
    card_id: i32,
    ad_data: AdData,
    price_data: Price,
    status: i32,
}

impl CardResponse {
    fn empty() -> CardResponse {
        CardResponse {
            card_id: 0,
            ad_data: AdData::empty(),
            price_data: Price::empty(),
            status: 0,
        }
    }
}

pub struct RentalData {
    pub rent: i32,
    pub size: f32,
}

#[derive(Debug)]
pub struct Oikotie {
    pub tokens: Option<Box<OikotieTokens>>,
}

impl Clone for Oikotie {
    fn clone(&self) -> Self {
        Oikotie {
            tokens: self.tokens.clone(),
        }
    }
}

impl Oikotie {
    pub async fn new() -> Oikotie {
        Oikotie {
            tokens: get_tokens().await,
        }
    }

    /// Use Oikotie's search API to find location ID based on text query.
    pub async fn get_locations_for_zip_code(
        &mut self,
        zip_code: &str,
    ) -> Result<Vec<LocationResponse>> {
        let locations = match fetch_location_id(self.tokens.as_ref().unwrap(), zip_code).await {
            Ok(l) => l,
            Err(e) => {
                error!("Error while fetching location id from Oikotie: {}", e);
                return Err(e.into());
            }
        };

        if locations.is_empty() {
            return Err(anyhow!(
                "Did not find any valid location for '{}', please try again!",
                zip_code
            ));
        }

        Ok(locations)
    }

    /// Fecthes all apartments for a certain location.
    pub async fn get_apartments(
        &mut self,
        config: Arc<Config>,
        watchlist: &Watchlist,
        size: SizeTarget,
    ) -> Result<Vec<InsertableApartment>> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        // TODO: Benchmark this function. Why so slow?
        let location: &Location = &Location {
            id: watchlist.location_id,
            level: watchlist.location_level,
            name: watchlist.location_name.clone(),
        };

        let cards_response: Result<CardsResponse> =
            fetch_apartments_for_sale(self.tokens.as_ref().unwrap(), location.clone(), size).await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(e) => return Err(e),
        };

        let mut apartments: Vec<InsertableApartment> = Vec::new();

        for card in cards {
            let existing_apartment =
                match db::apartment::get_apartment_by_card_id(&config, card.id.try_into().unwrap())
                {
                    Ok(ap) => ap,
                    Err(e) => return Err(e.into()),
                };

            let has_been_sent = has_been_sent_to_watchlist(config.clone(), &card, watchlist);

            if existing_apartment.is_none() && !has_been_sent {
                let apartment: InsertableApartment =
                    card_into_complete_apartment(self.tokens.as_ref().unwrap(), &card, location)
                        .await;
                apartments.push(apartment);
            }
        }

        Ok(apartments)
    }

    /// Fecthes all rental apartments for a certain location
    pub async fn get_rental_data(
        &mut self,
        location: &Location,
        size_range: SizeTarget,
    ) -> Result<Vec<RentalData>> {
        let location: Location = Location {
            id: location.id,
            level: location.level,
            name: location.name.clone(),
        };

        let oikotie_rental_cards_response: Result<CardsResponse> =
            fetch_apartments_for_rent(self.tokens.as_ref().unwrap(), location.clone(), size_range)
                .await;

        let oikotie_rental_cards = match oikotie_rental_cards_response {
            Ok(c) => c.cards,
            Err(_e) => return Err(anyhow!("Error while fetching cards")),
        };

        let oikotie_rental_cards_iter: std::slice::Iter<'_, Card> = oikotie_rental_cards.iter();
        let mut rents: Vec<RentalData> = Vec::new();

        for card in oikotie_rental_cards_iter {
            let rent = get_rent_regex(card.price.clone());
            let rent_data = RentalData {
                rent,
                size: card.size,
            };
            rents.push(rent_data);
        }

        Ok(rents)
    }

    /// Calculates the estimated rent fo the given apartment
    ///
    /// Depends on a call to Oikotie to get the nearby rental apartments.
    /// Estimated the rent using heuristics.
    /// TODO: Make rent evaluation ML based.
    pub async fn get_estimated_rent(
        &mut self,
        apartment: &InsertableApartment,
    ) -> Result<i32, Error> {
        let location = &Location {
            id: apartment.location_id.unwrap(),
            level: apartment.location_level.unwrap(),
            name: apartment.location_name.clone().unwrap(),
        };
        let size = apartment.size.unwrap_or_default();
        let size_range = SizeTarget {
            min: Some((size * 0.9) as i32),
            max: Some((size * 1.1) as i32),
        };

        let rental_apartments_nearby = self.get_rental_data(location, size_range).await;

        match rental_apartments_nearby {
            Ok(rental_data) => Ok(estimate_rent(size as f32, rental_data)),
            Err(e) => Err(anyhow!(
                "PRODUCER ERROR while calculating rent: {}",
                e.to_string()
            )),
        }
    }
}

async fn fetch_location_id(
    tokens: &OikotieTokens,
    zip_code: &str,
) -> Result<Vec<LocationResponse>, reqwest::Error> {
    // Use location level 5 here to get ZIP CODE locations
    let params: Vec<(&str, &str)> = vec![("query", zip_code), ("card_type", "5")];

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

    let response = send_request(RequestType::GET, URLS::LOCATION, params, headers).await;
    // Perform the actual request

    let api_response: Vec<LocationResponse> = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    Ok(api_response)
}

async fn fetch_card(
    tokens: &OikotieTokens,
    card_id: String,
) -> Result<CardResponse, reqwest::Error> {
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

    let api_response: CardResponse = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    Ok(api_response)
}

async fn fetch_apartments_for_sale(
    tokens: &OikotieTokens,
    location: Location,
    target_size: SizeTarget,
) -> Result<CardsResponse> {
    fetch_apartments(tokens, location, target_size, String::from(CardTypes::SELL)).await
}

async fn fetch_apartments_for_rent(
    tokens: &OikotieTokens,
    location: Location,
    target_size: SizeTarget,
) -> Result<CardsResponse> {
    fetch_apartments(tokens, location, target_size, String::from(CardTypes::RENT)).await
}

async fn fetch_apartments(
    tokens: &OikotieTokens,
    location: Location,
    target_size: SizeTarget,
    card_type: String,
) -> Result<CardsResponse> {
    let min_size = target_size.min.unwrap_or_default().to_string();
    let max_size = target_size.max.unwrap_or_default().to_string();
    let location = create_location_string(location.id, location.level, location.name);

    let mut params: Vec<(&str, &str)> = Vec::new();

    params.push(("cardType", &card_type));
    params.push(("locations", &location));

    // Add size requirements to query if given
    if !min_size.is_empty() {
        params.push(("size[min]", &min_size));
    }
    if !max_size.is_empty() {
        params.push(("size[max]", &max_size));
    }

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

    let response = send_request(
        crate::RequestType::GET,
        URLS::CARDS,
        params.clone(),
        headers.clone(),
    )
    .await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!(
            "HTTP Error: {} - {} for config - url: {}\n, params: {:?}\n headers: {:?}",
            status,
            error_text,
            URLS::CARDS,
            params,
            headers
        );
        return Err(anyhow!(
            "{} {}",
            reqwest::StatusCode::from_u16(status.as_u16()).unwrap(),
            format!("HTTP request failed with status {}", status),
        ));
    }

    let response_text = response.text().await?;
    if response_text.trim().is_empty() {
        error!("{}", "Empty response body");
    }

    let api_response = serde_json::from_str::<CardsResponse>(&response_text)?;
    Ok(api_response)
}

async fn card_into_complete_apartment(
    tokens: &OikotieTokens,
    card: &Card,
    location: &Location,
) -> InsertableApartment {
    // TODO FIX THIS TO HANDLE 5.0 API
    // Fetch card data that includes total price information
    let card_data: CardResponse = match fetch_card(tokens, card.id.to_string()).await {
        Ok(c) => c,
        Err(_e) => {
            error!(
                "Did not fetch card data for card {:?}. Error is {:?}",
                card.id, _e
            );
            CardResponse::empty()
        }
    };

    InsertableApartment {
        card_id: card.id as i32,
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
    }
}

fn has_been_sent_to_watchlist(config: Arc<Config>, card: &Card, watchlist: &Watchlist) -> bool {
    let apartments =
        match get_watchlist_apartment_connector(&config, watchlist, card.id.try_into().unwrap()) {
            Ok(aps) => aps,
            Err(_e) => {
                error!("Error while querying apartmetns withing period");
                vec![]
            }
        };

    if apartments.is_empty() {
        return false;
    }

    apartments[0].has_been_sent
}

fn price_int_or_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => {
            (num.as_f64().ok_or(de::Error::custom("Invalid number"))? as i32).to_string()
        }
        _ => return Err(de::Error::custom("wrong type")),
    })
}
