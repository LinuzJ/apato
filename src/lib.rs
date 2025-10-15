use models::{apartment::Apartment, watchlist::Watchlist};
use reqwest::header::HeaderMap;

pub mod bot;
pub mod config;
pub mod consumer;
pub mod db;
pub mod interest_rate;
pub mod logger;
pub mod ml_client;
pub mod models;
pub mod oikotie;
pub mod producer;

#[derive(Debug, Clone)]
pub enum TaskType {
    UpdateWatchlist,
    SendMessage,
}

#[derive(Clone)]
pub struct MessageTask {
    task_type: TaskType,
    watchlist: Watchlist,
    apartment: Option<Apartment>,
}

#[derive(PartialEq)]
pub enum RequestType {
    POST,
    GET,
}

pub struct URLS;

impl URLS {
    pub const CARDS: &str = "https://asunnot.oikotie.fi/api/5.0/cards";
    pub const LOCATION: &str = "https://asunnot.oikotie.fi/api/5.0/location";
}

pub async fn send_request(
    request_type: RequestType,
    url: &str,
    params: Vec<(&str, &str)>,
    headers: HeaderMap,
) -> Result<reqwest::Response, reqwest::Error> {
    let client: reqwest::Client = reqwest::Client::new();

    let request_builder = if request_type == RequestType::GET {
        client.get(url).query(&params)
    } else {
        client.post(url).json(&params)
    };

    let response = request_builder.headers(headers).send().await?;

    Ok(response)
}
