#[macro_use]
extern crate tokio;
extern crate chrono;
extern crate diesel;

pub mod bot;
pub mod config;
mod db;
mod interest_rate;
pub mod logger;
pub mod models;
mod oikotie;
pub mod producer;

pub use producer::calculate_irr;
