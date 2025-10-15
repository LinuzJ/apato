use std::sync::Arc;

use crate::config::Config;
use crate::db;
use crate::db::apartment_watchlist::get_watchlist_apartment_connector;
use crate::ml_client::{self, RentPredictionRequest};
use crate::models::apartment::InsertableApartment;
use crate::models::watchlist::SizeTarget;
use crate::models::watchlist::Watchlist;
use crate::oikotie::helpers;
use crate::oikotie::tokens;
use crate::send_request;
use crate::RequestType;
use crate::URLS;

use anyhow::anyhow;
use anyhow::{Context, Result};
use helpers::create_location_string;
use log::{error, warn};

use reqwest::header::{HeaderMap, HeaderValue};
use serde::de;
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    rooms: Option<u32>,
    #[serde(deserialize_with = "price_int_or_string")]
    price: String,
    #[serde(default)]
    published: Option<String>,
    #[serde(default, deserialize_with = "deserialize_f32_or_default")]
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
    #[serde(default, deserialize_with = "deserialize_u64_or_default")]
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
    #[serde(default, deserialize_with = "deserialize_u64_or_default")]
    maintenance_fee: u64,
    #[serde(default, deserialize_with = "deserialize_u64_or_default")]
    size: u64,
    #[serde(default)]
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

fn build_authenticated_headers(tokens: &OikotieTokens) -> Result<HeaderMap> {
    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(
        "ota-loaded",
        HeaderValue::from_str(&tokens.loaded)
            .context("Failed to set ota-loaded header from tokens")?,
    );
    headers.insert(
        "ota-cuid",
        HeaderValue::from_str(&tokens.cuid).context("Failed to set ota-cuid header from tokens")?,
    );
    headers.insert(
        "ota-token",
        HeaderValue::from_str(&tokens.token)
            .context("Failed to set ota-token header from tokens")?,
    );
    Ok(headers)
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

    async fn ensure_tokens(&mut self) -> Result<&OikotieTokens> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        self.tokens
            .as_deref()
            .ok_or_else(|| anyhow!("Failed to fetch authentication tokens from Oikotie"))
    }

    /// Use Oikotie's search API to find location ID based on text query.
    pub async fn get_locations_for_zip_code(
        &mut self,
        zip_code: &str,
    ) -> Result<Vec<LocationResponse>> {
        let tokens = self.ensure_tokens().await?;
        let locations = match fetch_location_id(tokens, zip_code).await {
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
        let tokens = self.ensure_tokens().await?.clone();

        // TODO: Benchmark this function. Why so slow?
        let location: &Location = &Location {
            id: watchlist.location_id,
            level: watchlist.location_level,
            name: watchlist.location_name.clone(),
        };

        let cards_response: Result<CardsResponse> =
            fetch_apartments_for_sale(&tokens, location.clone(), size).await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(e) => return Err(e),
        };

        let mut apartments: Vec<InsertableApartment> = Vec::new();

        for card in cards {
            let card_id: i32 = card
                .id
                .try_into()
                .map_err(|_| anyhow!("Card id {} does not fit in i32", card.id))?;

            let existing_apartment = match db::apartment::get_apartment_by_card_id(&config, card_id)
            {
                Ok(ap) => ap,
                Err(e) => return Err(e.into()),
            };

            let has_been_sent = has_been_sent_to_watchlist(config.clone(), watchlist, card_id)?;

            if existing_apartment.is_none() && !has_been_sent {
                let apartment: InsertableApartment =
                    card_into_complete_apartment(&tokens, &card, location).await?;
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
        let tokens = self.ensure_tokens().await?.clone();
        let location: Location = Location {
            id: location.id,
            level: location.level,
            name: location.name.clone(),
        };

        let oikotie_rental_cards_response: Result<CardsResponse> =
            fetch_apartments_for_rent(&tokens, location.clone(), size_range).await;

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
    /// Estimated the rent using heuristics or, if available, the external ML service.
    pub async fn get_estimated_rent(
        &mut self,
        config: &Arc<Config>,
        apartment: &InsertableApartment,
    ) -> Result<i32> {
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

        let rooms = apartment.rooms.unwrap_or_default();
        let price = apartment.price.unwrap_or_default();
        let maintenance_fee = apartment.additional_costs.unwrap_or_default();

        if config
            .ml_service_url
            .as_ref()
            .map(|url| !url.is_empty())
            .unwrap_or(false)
        {
            let request = RentPredictionRequest {
                location_id: location.id,
                location_level: location.level,
                size,
                rooms,
                price: price as f64,
                maintenance_fee: maintenance_fee as f64,
                auth_token: None,
            };

            match ml_client::predict_rent(config.as_ref(), request).await {
                Ok(prediction) if prediction > 0 => return Ok(prediction),
                Ok(_) => warn!(
                    "ML service returned non-positive rent for card {}, using heuristic fallback",
                    apartment.card_id
                ),
                Err(err) => warn!(
                    "Failed to fetch rent from ML service for card {}: {}",
                    apartment.card_id, err
                ),
            }
        }

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
) -> Result<Vec<LocationResponse>> {
    // Use location level 5 here to get ZIP CODE locations
    let params: Vec<(&str, &str)> = vec![("query", zip_code), ("card_type", "5")];

    let headers = build_authenticated_headers(tokens)?;

    let response = send_request(RequestType::GET, URLS::LOCATION, params, headers).await?;
    // Perform the actual request

    let api_response: Vec<LocationResponse> = response.json().await?;

    Ok(api_response)
}

async fn fetch_card(tokens: &OikotieTokens, card_id: String) -> Result<CardResponse> {
    let client: reqwest::Client = reqwest::Client::new();

    // Create request with needed token headers
    let mut oikotie_cards_api_url = String::from("https://asunnot.oikotie.fi/api/5.0/card/");
    oikotie_cards_api_url.push_str(&card_id.to_owned());

    let headers = build_authenticated_headers(tokens)?;

    // Perform the actual request
    let response = client
        .get(oikotie_cards_api_url)
        .headers(headers)
        .send()
        .await?;

    let api_response: CardResponse = response.json().await?;

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

    let headers = build_authenticated_headers(tokens)?;

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
) -> Result<InsertableApartment> {
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

    let card_id: i32 = card
        .id
        .try_into()
        .map_err(|_| anyhow!("Card id {} does not fit in i32", card.id))?;
    let price_i64 = i64::try_from(card_data.price_data.price)
        .context("Price does not fit in signed 64-bit integer")?;
    let maintenance_i64 = i64::try_from(card_data.ad_data.maintenance_fee)
        .context("Maintenance fee does not fit in signed 64-bit integer")?;

    let price = if price_i64 > i64::from(i32::MAX) {
        warn!(
            "Price {} exceeds i32 range for card {}, clamping to i32::MAX",
            price_i64, card_id
        );
        i32::MAX
    } else {
        price_i64 as i32
    };

    let maintenance_fee = if maintenance_i64 > i64::from(i32::MAX) {
        warn!(
            "Maintenance fee {} exceeds i32 range for card {}, clamping to i32::MAX",
            maintenance_i64, card_id
        );
        i32::MAX
    } else {
        maintenance_i64 as i32
    };

    Ok(InsertableApartment {
        card_id,
        location_id: Some(location.id),
        location_level: Some(location.level),
        location_name: Some(location.name.clone()),
        size: Some(card.size as f64),
        rooms: Some(card.rooms.unwrap_or_default() as i32),
        price: Some(price),
        additional_costs: Some(maintenance_fee),
        rent: Some(0),
        estimated_yield: Some(0.0),
        url: Some(card.url.clone()),
    })
}

fn has_been_sent_to_watchlist(
    config: Arc<Config>,
    watchlist: &Watchlist,
    card_id: i32,
) -> Result<bool> {
    let apartments = match get_watchlist_apartment_connector(&config, watchlist, card_id) {
        Ok(aps) => aps,
        Err(_e) => {
            error!(
                "Error while querying apartments within period for card {}: {:?}",
                card_id, _e
            );
            vec![]
        }
    };

    if apartments.is_empty() {
        return Ok(false);
    }

    Ok(apartments[0].has_been_sent)
}

fn price_int_or_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => {
            (num.as_f64().ok_or(de::Error::custom("Invalid number"))? as i32).to_string()
        }
        Value::Null => "0".to_string(),
        _ => return Err(de::Error::custom("wrong type")),
    })
}

fn deserialize_u64_or_default<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    let parsed = match value {
        Some(Value::Number(num)) => num.as_u64(),
        Some(Value::String(s)) => s
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .ok(),
        Some(Value::Null) | None => Some(0),
        other => {
            warn!("Unexpected value for u64 field from API: {:?}", other);
            Some(0)
        }
    };

    Ok(parsed.unwrap_or_else(|| {
        warn!(
            target: "apato::oikotie::payload",
            "Failed to parse u64 value from API payload, defaulting to 0"
        );
        0
    }))
}

fn deserialize_f32_or_default<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    let parsed = match value {
        Some(Value::Number(num)) => num.as_f64().map(|v| v as f32),
        Some(Value::String(s)) => s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
            .collect::<String>()
            .replace(',', ".")
            .parse::<f32>()
            .ok(),
        Some(Value::Null) | None => Some(0.0),
        other => {
            warn!("Unexpected value for f32 field from API: {:?}", other);
            Some(0.0)
        }
    };

    Ok(parsed.unwrap_or_else(|| {
        warn!(
            target: "apato::oikotie::payload",
            "Failed to parse f32 value from API payload, defaulting to 0"
        );
        0.0
    }))
}
