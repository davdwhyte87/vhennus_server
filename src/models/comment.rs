

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use strum_macros;



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Comment {
    pub id:String, 
    pub text:String,
    pub user_name:String,
    pub created_at:NaiveDateTime,
    pub post_id:String
}