use crate::helpers;
use crate::marketplace_client;
use crate::marketplace_client::Apartment;
use crate::tokens;
use helpers::create_location_string;
use marketplace_client::Location;
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

fn card_into_apartment(card: &Card) -> Apartment {
    Apartment {
        id: card.id.to_string(),
        location: Location {
            id: 123,
            level: 123,
            name: String::from("TODO"),
        },
        size: card.size as u16,
        rooms: card.rooms as u16,
        price: card.price.to_owned(),
        additional_costs: 0,
    }
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
        ("ApartmentType", if get_rentals { "101" } else { "100" }),
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

impl OikotieClient {
    pub async fn new() -> OikotieClient {
        OikotieClient {
            tokens: get_tokens().await,
        }
    }

    pub async fn get_apartments(mut self, location: Location) -> Vec<Apartment> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let cards_response = fetch_apartments(&self.tokens.unwrap(), location, true).await;

        let cards: Vec<Card> = match cards_response {
            Ok(c) => c.cards,
            Err(_e) => Vec::new(),
        };

        let apartmens: Vec<Apartment> = cards.iter().map(card_into_apartment).collect();
        apartmens
    }
}
