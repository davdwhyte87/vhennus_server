

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use strum_macros;



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Profile {
    pub id: String,
    pub user_name:String, 
    pub bio: Option<String>,
    pub name:Option<String>,
    pub image:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub app_f_token: Option<String> ,// app firebase token
    pub wallets: Option<String>,
    pub unclaimed_earnings:BigDecimal,
    pub is_earnings_activated:bool,
    pub referred_users: Vec<String>,
    pub earnings_wallet: Option<String>
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Friend{
    pub id:i32,
    pub user_username:String,
    pub friend_username:String,
}
