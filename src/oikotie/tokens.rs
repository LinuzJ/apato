use log::{error, info};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

use super::helpers::generate_random_number;

#[derive(Debug, Deserialize)]
struct User {
    cuid: String,
    token: String,
    time: u32,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    user: User,
}

#[derive(Debug)]
pub struct OikotieTokens {
    pub loaded: String,
    pub cuid: String,
    pub token: String,
}

async fn fetch_tokens() -> Result<Box<OikotieTokens>, reqwest::Error> {
    let client: reqwest::Client = reqwest::Client::new();

    info!("Fetching Oikotie tokens");

    let num: String = generate_random_number();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", &num)];
    let header_values = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static(header_values));

    let response: Result<reqwest::Response, reqwest::Error> = client
        .get("https://asunnot.oikotie.fi/user/get")
        .query(&params)
        .headers(headers)
        .send()
        .await;

    let api_response: ApiResponse = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    println!("Tokens: {:?}", api_response);

    let tokens: Box<OikotieTokens> = Box::new(OikotieTokens {
        loaded: api_response.user.time.to_string(),
        cuid: api_response.user.cuid,
        token: api_response.user.token,
    });

    Ok(tokens)
}

pub async fn get_tokens() -> Option<Box<OikotieTokens>> {
    let tokens: Result<Box<OikotieTokens>, reqwest::Error> = fetch_tokens().await;

    return match tokens {
        Ok(tokens) => Some(tokens),
        Err(_e) => {
            error!("Error while fetching oikotie tokens.. Error: {:?}", _e);
            return None;
        }
    };
}
