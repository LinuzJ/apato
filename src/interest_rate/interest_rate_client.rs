use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};

use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct ApiResponse {
    mortgages: Vec<Mortgage>,
}

#[derive(Debug, Deserialize)]
struct Mortgage {
    interest_rate: f64,
}

pub async fn get_interest_rate() -> Result<f64, reqwest::Error> {
    // Read config file
    let config_file_path = "src/interest_rate/loan_interest_confis.json";
    let file = fs::File::open(config_file_path).expect("file should open read only");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");
    let loan_duration_years = json
        .get("loan_duration_years")
        .expect("file should have loan_duration_years key")
        .as_i64()
        .expect("Invalid loan_duration_years");
    let down_payment_percentage = json
        .get("down_payment_percentage")
        .expect("file should have down_payment_percentage key")
        .as_f64()
        .expect("Invalid down_payment_percentage");

    // Values used for request
    let price: i64 = 200000;
    let down_payment = price * down_payment_percentage as i64;
    let months = loan_duration_years * 12;

    let nordea_url = "https://hj.nordea.com/hj/common/api/wdamc/nordic/products/calculate";
    let nordea_loan_api_url = String::from(nordea_url);
    let client: reqwest::Client = Client::new();

    let json_body = &json!({
        "amortization_type": "ANNUITY",
        "country": "FI",
        "down_payment": down_payment,
        "duration_in_months": months,
        "estimated_property_value": price,
        "finland_only_input": {
            "first_time_buyer": false
        },
        "individual_pricing": false,
        "interest_only_period_in_months": 0,
        "loan_product_id": "06dce690-9d4a-41db-9e8e-62bccd84486f",
        "payment_day": "2023-06-27",
    });

    let request = client.post(nordea_loan_api_url).json(json_body);

    println!("REQUEST: {:?}", request);

    let response = request.send().await?;

    println!("REPOSENSE: {:?}", response);
    let api_response: ApiResponse = response.json().await?;

    Ok(api_response.mortgages[0].interest_rate)
}
