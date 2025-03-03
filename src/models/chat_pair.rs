
use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::{self, ToString}};
use chrono::NaiveDateTime;
use strum_macros;



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatPair {
    pub id: String,
    pub user1:String,
    pub user2:String,
    pub last_message:Option<String>,
    pub all_read: bool,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}
