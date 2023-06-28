#[macro_use]
extern crate rocket;

use rand::Rng;
use reqwest::Client;
use reqwest::Response;
use reqwest::header::{HeaderMap, HeaderValue};

struct OikotieTokens {
    loaded: String,
    cuid: String,
    token: String,
}

async fn fetch_oikotie_tokens() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let random_number: String = rand
        ::thread_rng()
        .gen_range(5000..10000)
        .to_string();
    let params: Vec<(&str, &str)> = vec![("format", "json"), ("rand", &random_number)];

    let mut headers = HeaderMap::new();
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"));

    let response = client
        .get("https://asunnot.oikotie.fi/user/get")
        .query(&params)
        .headers(headers)
        .send()
        .await?;

    // Process the response as needed
    println!("Response status: {}", response.status());
    let body = response.text().await?;
    println!("Response body:\n{}", body);

    Ok(())
}

#[get("/")]
async fn index() -> &'static str {
    let tokens = fetch_oikotie_tokens().await;
    "heh"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index])
        .launch()
        .await?;

    Ok(())
}
