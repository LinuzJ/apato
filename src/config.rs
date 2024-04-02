use dotenvy::dotenv;
use log::error;
use serde::Deserialize;
use std::env;

const CONFIG_PATH_ENV: &str = "CONFIG_PATH";

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub db_path: String,
    pub telegram_bot_token: String,
    pub loan_duration_years: u32,
    pub down_payment_percentage: u32,
    pub avg_vacant_month_per_year: u32,
    pub avg_estimated_rent_increase_per_year: u32,
    pub estimated_yearly_apartment_price_increase: u32,
    pub avg_renovation_costs: u32,
    pub tax: u32,
}

pub fn create_test_config() -> Config {
    Config {
        db_path: "xxx".to_string(),
        telegram_bot_token: "xxx".to_string(),
        loan_duration_years: 25,
        down_payment_percentage: 20,
        avg_vacant_month_per_year: 1,
        avg_estimated_rent_increase_per_year: 1,
        estimated_yearly_apartment_price_increase: 2,
        avg_renovation_costs: 5000,
        tax: 30,
    }
}

pub fn read_config() -> Config {
    dotenv().ok();
    env::var(CONFIG_PATH_ENV)
        .map_err(|_| format!("{CONFIG_PATH_ENV} .env not set"))
        .and_then(|config_path| std::fs::read(config_path).map_err(|e| e.to_string()))
        .and_then(|bytes| toml::from_slice(&bytes).map_err(|e| e.to_string()))
        .unwrap_or_else(|err| {
            error!("failed to read config: {err}");
            std::process::exit(1);
        })
}
