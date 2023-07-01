use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

use crate::helpers::generate_random_number;

#[derive(Debug, Deserialize)]
struct User {
    cuid: Box<str>,
    token: Box<str>,
    time: Box<str>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    user: User,
}

pub struct OikotieTokens {
    pub loaded: Box<Box<str>>,
    pub cuid: Box<Box<str>>,
    pub token: Box<Box<str>>,
}

async fn fetch_tokens() -> Result<Box<OikotieTokens>, reqwest::Error> {
    let client: reqwest::Client = reqwest::Client::new();

    let num: String = generate_random_number();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", &num)];

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"));

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

    let tokens: Box<OikotieTokens> = Box::new(OikotieTokens {
        loaded: Box::new(api_response.user.time),
        cuid: Box::new(api_response.user.cuid.to_owned()),
        token: Box::new(api_response.user.token.to_owned()),
    });

    Ok(tokens)
}

pub async fn get_tokens() -> Box<OikotieTokens> {
    let tokens: Result<Box<OikotieTokens>, reqwest::Error> = fetch_tokens().await;

    return match tokens {
        Ok(tokens) => tokens,
        Err(_e) => Box::new(OikotieTokens {
            loaded: todo!(),
            cuid: todo!(),
            token: todo!(),
        }),
    };
}
