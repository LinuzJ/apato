use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use rand::Rng;

#[derive(Debug, Deserialize)]
struct User {
    cuid: String,
    token: String,
    time: u64,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    user: User,
}

pub struct OikotieTokens {
    pub loaded: String,
    pub cuid: String,
    pub token: String,
}

async fn fetch_tokens() -> Result<OikotieTokens, reqwest::Error> {
    let client = reqwest::Client::new();
    let random_number: String = rand
        ::thread_rng()
        .gen_range(5000..10000)
        .to_string();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", &random_number)];

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"));

    let response = client
        .get("https://asunnot.oikotie.fi/user/get")
        .query(&params)
        .headers(headers)
        .send()
        .await?;

    let api_response: ApiResponse = response.json().await?;

    let tokens: OikotieTokens = OikotieTokens {loaded: api_response.user.time.to_string(), cuid: api_response.user.cuid, token: api_response.user.token};

    Ok(tokens)
}

pub async fn get_tokens() -> OikotieTokens {
    let tokens: Result<OikotieTokens, reqwest::Error> = fetch_tokens().await;
    return match tokens {
        Ok(tokens) => tokens,
        Err(e) => OikotieTokens{ loaded: todo!(), cuid: todo!(), token: todo!() }
    };
}