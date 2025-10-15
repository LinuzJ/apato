use anyhow::{anyhow, Context, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Serialize)]
pub struct RentPredictionRequest<'a> {
    pub location_id: i32,
    pub location_level: i32,
    pub size: f64,
    pub rooms: i32,
    pub price: f64,
    pub maintenance_fee: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct RentPredictionResponse {
    rent: i32,
}

pub async fn predict_rent(config: &Config, payload: RentPredictionRequest<'_>) -> Result<i32> {
    let base_url = config
        .ml_service_url
        .as_ref()
        .ok_or_else(|| anyhow!("ML service URL not configured"))?;

    let client = reqwest::Client::new();
    let url = format!("{}/predict", base_url.trim_end_matches('/'));

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .context("Failed to reach ML prediction service")?;

    if response.status() == StatusCode::NOT_FOUND {
        return Err(anyhow!("ML prediction endpoint not found (404)"));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "ML service responded with error {}: {}",
            status,
            body
        ));
    }

    let parsed: RentPredictionResponse = response
        .json()
        .await
        .context("Failed to deserialize ML prediction response")?;

    Ok(parsed.rent)
}
