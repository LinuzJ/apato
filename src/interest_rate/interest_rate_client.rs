use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    mortgages: Vec<Mortgage>,
}

#[derive(Debug, Deserialize)]
struct Mortgage {
    interest_rate: f64,
}

pub async fn get_interest_rate() -> Result<f64, reqwest::Error> {
    let nordea_loan_api_url =
        String::from("https://hj.nordea.com/hj/common/api/wdamc/nordic/products/calculate");
    let client: reqwest::Client = Client::new();
    let response = client.post(nordea_loan_api_url).send().await?;

    let api_response: ApiResponse = response.json().await?;
    Ok(api_response.mortgages[0].interest_rate)
}
