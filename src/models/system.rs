

use chrono::format::Numeric;
use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct System {
    pub id: i32,
    pub price: BigDecimal, // usd
    pub android_app_version:String,
    pub trivia_win_amount:BigDecimal,
    pub apk_link:String,
    pub ngn:BigDecimal,
}
