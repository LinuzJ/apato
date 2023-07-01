use std::f32::consts::E;

use crate::helpers;
use crate::marketplace_client;
use crate::marketplace_client::Apartment;
use crate::tokens;
use helpers::create_location_string;
use marketplace_client::{Location, MarketplaceClient};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokens::{get_tokens, OikotieTokens};

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    id: u32,
    url: String,
    description: String,
    rooms: u8,
    price: String,
    published: String,
    size: f32,
    sizeLot: u32,
    status: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct OitkotieCardsApiResponse {
    found: u32,
    cards: Vec<Card>,
}

pub struct OikotieClient {
    pub tokens: Option<Box<OikotieTokens>>,
}

fn fetch_apartments(
    tokens: &OikotieTokens,
    location: Location,
) -> Result<OitkotieCardsApiResponse, reqwest::Error> {
    let oikotie_cards_api_url = "https://asunnot.oikotie.fi/api/cards";

    let client: reqwest::blocking::Client = reqwest::blocking::Client::new();
    let locations: String = create_location_string(location.id, location.level, location.name);
    let params: Vec<(&str, &str)> = vec![("ApartmentType", "100"), ("locations", &locations)];
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
        .send();

    let api_response: OitkotieCardsApiResponse = match response {
        Ok(re) => re.json()?,
        Err(e) => return Err(e),
    };

    let cards = OitkotieCardsApiResponse {
        found: api_response.found,
        cards: api_response.cards,
    };

    return Ok(cards);
}

impl MarketplaceClient for OikotieClient {
    fn get_apartments(mut self, location: Location) -> Vec<Apartment> {
        if self.tokens.is_none() {
            self.tokens = Some(get_tokens());
        }

        let cards_response = fetch_apartments(&self.tokens.unwrap(), location);

        let cards: Vec<Card> = match cards_response {
            Ok(c) => c.cards,
            Err(e) => Vec::new(),
        };

        println!("{:?}", cards);
        let apartmens: Vec<Apartment> = Vec::new();
        apartmens
    }
}

// What do I need
// https://asunnot.oikotie.fi/api/cards?ApartmentType=100&locations=[[1652,4,"Taka-Töölö, Helsinki"]]

// Client needs to be able to:
// Search for apartments -> cards, type 100, location specified
// Fetch data for the found apartments
// Return that data
