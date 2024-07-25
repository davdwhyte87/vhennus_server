use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::{buy_order::BuyOrder, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SellOrder {
    pub id: String,
    pub user_name: String,
    pub buy_orders_id:Vec<String>,
    pub buy_orders: Option<Vec<BuyOrder>>,
    pub amount:BigDecimal,
    pub min_amount:BigDecimal,
    pub max_amount:BigDecimal,
    pub is_cancelled:bool,
    pub currency:Currency,
    pub created_at:String,
    pub updated_at: Option<String>
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq )]
pub enum Currency{
    NGN,
    USD,
    BTC,
    XRP
}