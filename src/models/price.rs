

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;




#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Price {
    pub id: String,
    pub created_at:String,
    pub price:BigDecimal
}