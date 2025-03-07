use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;
use crate::models::buy_order::BuyOrder;
use crate::models::payment_method::{PaymentMethod, PaymentMethodData};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SellOrder {
    pub id: String,
    pub user_name: String,
    pub buy_orders_id:Vec<String>,
    pub buy_orders: Option<Vec<BuyOrder>>,
    pub amount:BigDecimal,
    pub price: BigDecimal,
    pub min_amount:BigDecimal,
    pub max_amount:BigDecimal,
    pub is_closed:bool,
    pub currency:Currency,
    pub created_at:String,
    pub updated_at: Option<String>,
    pub payment_method: PaymentMethod,
    pub payment_method_id: String,
    pub payment_method_data: Option<PaymentMethodData>,
    pub wallet_address:String,
    pub phone_number:Option<String>
}



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,strum_macros::Display, Default )]
pub enum Currency{
    #[default]
    NGN,
    USD,
    BTC,
    XRP
}