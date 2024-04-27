use models::{apartment::Apartment, watchlist::Watchlist};

pub mod bot;
pub mod config;
pub mod consumer;
pub mod db;
pub mod interest_rate;
pub mod logger;
pub mod models;
pub mod oikotie;
pub mod producer;

#[derive(Debug, Clone)]
pub enum TaskType {
    UpdateWatchlist,
    SendMessage,
}

#[derive(Clone)]
pub struct MessageTask {
    task_type: TaskType,
    watchlist: Watchlist,
    apartment: Option<Apartment>,
}
