use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LoanData {
    mortgages: Vec<Mortgage>,
}

#[derive(Debug, Deserialize)]
struct Mortgage {
    interest_rate: f64,
}

pub async fn get_interest_rate() -> Result<f64, reqwest::Error> {
    let mut nordea_loan_api_url =
        String::from("https://hj.nordea.com/hj/common/api/wdamc/nordic/products/calculate");
    let client: reqwest::Client = Client::new();
    let response = client.post(nordea_loan_api_url).send().await;

    let api_response: LoanData = match response {
        Ok(re) => re.json().await?,
        Err(e) => return Err(e),
    };

    Ok(api_response.mortgages[0].interest_rate)
}
