

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use strum_macros;



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Post {
    pub id: String,
    pub text:String, 
    pub image: Option<String>, 
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub user_name:String,
}


