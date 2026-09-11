use std::collections::HashMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct System {
    pub id: i32,
    pub price: BigDecimal, // usd
    pub android_app_version: String,
    pub trivia_win_amount: BigDecimal,
    pub apk_link: String,
    pub ngn: BigDecimal,
    pub price_per_min: BigDecimal,
    pub ref_amount: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LiveRateResponse {
    pub success: bool,
    pub terms: String,
    pub privacy: String,
    pub timestamp: i64,
    pub source: String,
    pub quotes: HashMap<String, f64>,
}