use crate::db::schema::apartments::watchlist_id;
use crate::models::apartment::Apartment;
use crate::models::watchlist::Watchlist;
use crate::oikotie::helpers;
use crate::oikotie::tokens;

use helpers::create_location_string;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokens::{get_tokens, OikotieTokens};

#[derive(Debug)]
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
    // rooms: f32,
    published: String,
    size: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Price {
    myyntihinta: String,
    rahoitusvastike: String,
    hoitovastike: String,
    yhtiövastike: String,
    velkaosuus: String,
}

impl Price {
    fn empty() -> Price {
        Price {
            myyntihinta: String::from(""),
            rahoitusvastike: String::from(""),
            hoitovastike: String::from(""),
            yhtiövastike: String::from(""),
            velkaosuus: String::from(""),
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
    id: String,
    price: u16,
    size: f32,
    room_configuration: String,
    price_data: Price,
    status: u16,
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

async fn card_into_complete_apartment(
    tokens: &OikotieTokens,
    card: &Card,
    current_watchlist_id: i32,
) -> Apartment {
    let card_data = match fetch_card(tokens, card.id.to_string()).await {
        Ok(c) => c,
        Err(_e) => OitkotieCardApiResponse {
            id: String::from(""),
            price: 0,
            size: 0.0,
            room_configuration: String::from(""),
            price_data: Price::empty(),
            status: 0,
        },
    };
    Apartment {
        card_id: card.id.to_string(),
        location_id: 123,
        location_level: 123,
        location_name: String::from("TODO"),
        size: card.size as f64,
        // rooms: card.rooms as i32,
        price: card_data.price.to_string(),
        additional_costs: 0,
        rent: 0,
        watchlist_id: current_watchlist_id,
    }
}

impl OikotieClient {
    pub async fn new() -> OikotieClient {
        OikotieClient {
            tokens: get_tokens().await,
        }
    }

    pub async fn get_apartments(
        mut self,
        watchlist: Watchlist,
        get_rentals: bool,
    ) -> Option<Vec<Apartment>> {
        if self.tokens.is_none() {
            self.tokens = get_tokens().await;
        }

        let location: Location = Location {
            id: watchlist.id,
            level: watchlist.location_level,
            name: watchlist.location_name,
        };

        let cards_response: Result<OitkotieCardsApiResponse, reqwest::Error> =
            fetch_apartments(&self.tokens.as_ref().unwrap(), location, get_rentals).await;

        let cards = match cards_response {
            Ok(c) => c.cards,
            Err(_e) => return None,
        };

        let mut cards_iter: std::slice::Iter<'_, Card> = cards.iter();
        let mut apartments: Vec<Apartment> = Vec::new();

        while let Some(card) = cards_iter.next() {
            let apartment =
                card_into_complete_apartment(&self.tokens.as_ref().unwrap(), card, watchlist.id)
                    .await;
            apartments.push(apartment);
        }

        return Some(apartments);
    }
}
