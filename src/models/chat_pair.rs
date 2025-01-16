


use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::{self, ToString}};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, profile::Profile, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChatPair {
    pub id: String,
    pub user_name:String,
    pub users_ids:Vec<String>,
    pub users:Option<Vec<Profile>>,
    pub last_message:String,
    pub all_read: bool,
    pub created_at:String,
    pub updated_at:String,
}
