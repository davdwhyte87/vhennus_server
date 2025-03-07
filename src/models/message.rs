

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;




#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OrderMessage {
    pub id: String,
    pub text:String, 
    pub image: String, 
    pub created_at:String,
    pub sender_user_name:String,
    pub receiver_user_name:String,
    pub buy_order_id:String
}
