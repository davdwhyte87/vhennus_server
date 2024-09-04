use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::user::UserType;



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyOrder {
    pub id: String,
    pub user_name: String,
    pub amount:BigDecimal,
    pub sell_order_id: String,
    pub is_seller_confirmed:bool,
    pub is_buyer_confirmed:bool,
    pub is_canceled:bool,
    pub is_reported:bool,
    pub created_at:String, 
    pub updated_at: String,
    pub wallet_address:String
}