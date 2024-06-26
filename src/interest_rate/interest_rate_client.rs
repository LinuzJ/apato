use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    mortgages: Vec<Mortgage>,
}

#[derive(Debug, Deserialize)]
struct Mortgage {
    interest_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct NordeaRequestData {
    amortization_type: String,
    country: String,
    down_payment: i64,
    duration_in_months: i64,
    estimated_property_value: i32,
    finland_only_input: FinlandInput,
    individual_pricing: bool,
    interest_only_period_in_months: i32,
    loan_product_id: String,
    payment_day: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FinlandInput {
    first_time_buyer: bool,
}

pub async fn get_interest_rate(config: &Arc<Config>) -> Result<f64, reqwest::Error> {
    let loan_duration_years: i32 = config.loan_duration_years as i32;
    let down_payment_percentage: f32 = config.down_payment_percentage as f32 / 100.0;

    // Values used for request
    let price: i64 = 200000;
    let down_payment = (price as f32 * down_payment_percentage) as i64;
    let months = loan_duration_years * 12;

    let amortization_type = String::from("ANNUITY");
    let country = String::from("FI");
    let down_payment: i64 = down_payment;
    let duration_in_months: i64 = months as i64;
    let estimated_property_value = 200000;
    let finland_only_input = FinlandInput {
        first_time_buyer: false,
    };
    let individual_pricing = false;
    let interest_only_period_in_months = 0;
    let loan_product_id = String::from("06dce690-9d4a-41db-9e8e-62bccd84486f"); // Not a token :)
    let payment_day = String::from("2023-10-27");

    // Create a RequestData object
    let json_body = NordeaRequestData {
        amortization_type,
        country,
        down_payment,
        duration_in_months,
        estimated_property_value,
        finland_only_input,
        individual_pricing,
        interest_only_period_in_months,
        loan_product_id,
        payment_day,
    };

    let nordea_url = "https://hj.nordea.com/hj/common/api/wdamc/nordic/products/calculate";
    let nordea_loan_api_url = String::from(nordea_url);
    let client: Client = Client::new();
    let request = client.post(nordea_loan_api_url).json(&json_body);

    let response = request.send().await?;
    let api_response: ApiResponse = response.json().await?;

    Ok(api_response.mortgages[0].interest_rate)
}
