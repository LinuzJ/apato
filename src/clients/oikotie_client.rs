use crate::clients::helpers;
use crate::clients::tokens;
use crate::modules::apartment::Apartment;
use helpers::create_location_string;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokens::{get_tokens, OikotieTokens};

#[derive(Debug)]
pub struct Location {
    pub id: u16,
    pub level: u8,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    id: u32,
    url: String,
    description: String,
    rooms: u8,
    published: String,
    size: f32,
    status: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    Myyntihinta: String,
    Rahoitusvastike: String,
    Hoitovastike: String,
    Yhtiövastike: String,
    Velkaosuus: String,
}

impl Price {
    fn empty() -> Price {
        Price {
            Myyntihinta: String::from(""),
            Rahoitusvastike: String::from(""),
            Hoitovastike: String::from(""),
            Yhtiövastike: String::from(""),
            Velkaosuus: String::from(""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OitkotieCardsApiResponse {
    found: u32,
    cards: Vec<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OitkotieCardApiResponse {
    id: String,
    price: u16,
    size: f32,
    roomConfiguration: String,
    priceData: Price,
}

#[derive(Debug)]
pub struct OikotieClient {
    pub tokens: Option<Box<OikotieTokens>>,
}

async fn fetch_card(
    tokens: &OikotieTokens,
    card_id: String,
) -> Result<OitkotieCardApiResponse, reqwest::Error> {
    let client: reqwest::Client = reqwest::Client::new();

    let mut oikotie_cards_api_url = String::from("https://asunnot.oikotie.fi/api/card/");
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

async fn card_into_apartment(tokens: &OikotieTokens, card: &Card) -> Apartment {
    let card_data = match fetch_card(tokens, card.id.to_string()).await {
        Ok(c) => c,
        Err(_e) => OitkotieCardApiResponse {
            id: String::from(""),
            price: 0,
            size: 0.0,
            roomConfiguration: String::from(""),
            priceData: Price::empty(),
        },
    };
    Apartment {
        id: card.id.to_string(),
        location_id: 123,
        location_level: 123,
        location_name: String::from("TODO"),
        size: card.size as f64,
        rooms: card.rooms as i32,
        price: card_data.price.to_string(),
        additional_costs: 0,
    }
}

impl OikotieClient {
    pub async fn new() -> OikotieClient {
        OikotieClient {
            tokens: get_tokens().await,
        }
    }

    pub async fn get_apartments(mut self, location: Location, get_rentals: bool) -> Vec<Apartment> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let cards_response: Result<OitkotieCardsApiResponse, reqwest::Error> =
            fetch_apartments(&self.tokens.as_ref().unwrap(), location, get_rentals).await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(_e) => Vec::new(),
        };

        let mut cards_iter: std::slice::Iter<'_, Card> = cards.iter();
        let mut apartments: Vec<Apartment> = Vec::new();

        while let Some(card) = cards_iter.next() {
            let apartment = card_into_apartment(&self.tokens.as_ref().unwrap(), card).await;
            apartments.push(apartment);
        }

        apartments
    }
}
