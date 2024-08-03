use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::string::ToString;
use strum_macros;

use super::{buy_order::BuyOrder, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellOrder {
    pub id: String,
    pub user_name: String,
    pub buy_orders_id:Vec<String>,
    pub buy_orders: Option<Vec<BuyOrder>>,
    pub amount:BigDecimal,
    pub min_amount:BigDecimal,
    pub max_amount:BigDecimal,
    pub is_closed:bool,
    pub currency:Currency,
    pub created_at:String,
    pub updated_at: Option<String>,
    pub payment_method: PaymentMethod,
    pub payment_method_id: String,
  
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,strum_macros::Display )]
pub enum Currency{
    NGN,
    USD,
    BTC,
    XRP
}