use crate::models::apartment::Apartment;
use crate::models::watchlist::Watchlist;
use crate::oikotie::helpers;
use crate::oikotie::tokens;

use helpers::create_location_string;
use log::error;
use reqwest::header::{HeaderMap, HeaderValue};
use rocket::Error;
use serde::{Deserialize, Serialize};
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
struct Card {
    id: u32,
    url: String,
    description: String,
    rooms: u32,
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
) -> Apartment {
    if is_handling_rent {
        // Because rent is in weird string format -> regex match to i32
        let rent = get_rent_regex(card.price.clone());

        return Apartment {
            card_id: card.id.to_string(),
            location_id: location.id,
            location_level: location.level,
            location_name: location.name.clone(),
            size: card.size as f64,
            rooms: card.rooms as i32,
            price: 0,
            additional_costs: 0,
            rent: rent,
            watchlist_id: -1,
        };
    }

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

    Apartment {
        card_id: card.id.to_string(),
        location_id: location.id,
        location_level: location.level,
        location_name: location.name.clone(),
        size: card.size as f64,
        rooms: card.rooms as i32,
        price: card_data.price_data.price as i32,
        additional_costs: 0,
        rent: 0,
        watchlist_id: watchlist_id,
    }
}

impl Oikotie {
    pub async fn new() -> Oikotie {
        Oikotie {
            tokens: get_tokens().await,
        }
    }

    /*
       Fecthes all apartments for a certain location
    */
    pub async fn get_apartments(&mut self, watchlist: &Watchlist) -> Option<Vec<Apartment>> {
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
        let mut apartments: Vec<Apartment> = Vec::new();

        while let Some(card) = cards_iter.next() {
            let apartment: Apartment = card_into_complete_apartment(
                &self.tokens.as_ref().unwrap(),
                card,
                location,
                Some(watchlist.id),
                is_handling_rent,
            )
            .await;
            apartments.push(apartment);
        }

        return Some(apartments);
    }

    /*
       Fecthes all rental apartments for a certain location
    */
    pub async fn get_rental_apartments(&mut self, location: &Location) -> Option<Vec<Apartment>> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let is_handling_rent = true;

        let location: Location = Location {
            id: location.id,
            level: location.level,
            name: location.name.clone(),
        };

        let cards_response: Result<OitkotieCardsApiResponse, reqwest::Error> =
            fetch_apartments(&self.tokens.as_ref().unwrap(), location.clone(), true).await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(_e) => return None,
        };

        let mut cards_iter: std::slice::Iter<'_, Card> = cards.iter();
        let mut apartments: Vec<Apartment> = Vec::new();

        while let Some(card) = cards_iter.next() {
            let apartment = card_into_complete_apartment(
                &self.tokens.as_ref().unwrap(),
                card,
                &location,
                None,
                is_handling_rent,
            )
            .await;
            apartments.push(apartment);
        }

        return Some(apartments);
    }

    /*
       Calculates and returns the estimated rent for a given location
    */
    pub async fn get_estimated_rent(&mut self, apartment: &Apartment) -> Result<i32, Error> {
        let location = &Location {
            id: apartment.location_id,
            level: apartment.location_level,
            name: apartment.location_name.clone(),
        };
        let rental_apartments = self.get_rental_apartments(location).await;

        match rental_apartments {
            Some(apartments_with_rent) => {
                let rent: i32 = estimated_rent(apartment, apartments_with_rent);
                return Ok(rent);
            }
            None => panic!("Error while calculating rent"),
        }
    }
}
