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

pub struct OikotieClient<'a> {
    pub tokens: &'a OikotieTokens<'a>,
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

    match HeaderValue::from_str(tokens.loaded) {
        Ok(header) => headers.insert("ota-loaded", header),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(tokens.cuid) {
        Ok(header) => headers.insert("ota-cuid", header),
        Err(_e) => todo!(),
    };
    match HeaderValue::from_str(tokens.token) {
        Ok(header) => headers.insert("ota-token", header),
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

impl MarketplaceClient for OikotieClient<'_> {
    fn set_tokens(mut self) {
        self.tokens = &get_tokens();
    }

    fn get_apartments(mut self, location: Location) -> Vec<Apartment> {
        let cards_response = fetch_apartments(self.tokens, location);

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
