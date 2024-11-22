

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::{self, ToString}};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Circle {
    pub id: String,
    pub name:String,
    pub display_name:String,
    pub owner:String,
    pub members:Vec<String>,
    pub image:String,
    pub is_private: bool,
    pub created_at:String,
    pub updated_at:String,
}
