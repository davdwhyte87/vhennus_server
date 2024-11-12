

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Profile {
    pub id: String,
    pub user_name:String, 
    pub bio: String,
    pub work:String,
    pub occupation:String, 
    pub image:String,
    pub created_at:String,
    pub updated_at:String,
    pub friends:Vec<String>,
    pub friends_models:Option<Vec<Profile>>
}
