

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use strum_macros;

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct Chat {
    pub id: String,
    pub sender:String,
    pub receiver:String,
    pub message:String,
    pub image:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub pair_id:String
}
