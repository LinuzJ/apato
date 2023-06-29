use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct User<'a> {
    cuid: &'a str,
    token: &'a str,
    time: &'a str,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<'a> {
    user: User<'a>,
}

pub struct OikotieTokens<'a> {
    pub loaded: &'a str,
    pub cuid: &'a str,
    pub token: &'a str,
}

fn generate_random_number() -> String {
    rand::thread_rng().gen_range(5000..10000).to_string()
}

fn fetch_tokens() -> Result<OikotieTokens<'static>, reqwest::Error> {
    let client: reqwest::blocking::Client = reqwest::blocking::Client::new();

    let num: String = generate_random_number();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", &num)];

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"));

    let response = client
        .get("https://asunnot.oikotie.fi/user/get")
        .query(&params)
        .headers(headers)
        .send();

    let api_response: ApiResponse = match response {
        Ok(re) => re.json()?,
        Err(e) => return Err(e),
    };

    let tokens: OikotieTokens = OikotieTokens {
        loaded: &api_response.user.time,
        cuid: api_response.user.cuid,
        token: api_response.user.token,
    };

    Ok(tokens)
}

pub fn get_tokens() -> OikotieTokens<'static> {
    let tokens: Result<OikotieTokens<'_>, reqwest::Error> = fetch_tokens();

    return match tokens {
        Ok(tokens) => tokens,
        Err(_e) => OikotieTokens {
            loaded: todo!(),
            cuid: todo!(),
            token: todo!(),
        },
    };
}
