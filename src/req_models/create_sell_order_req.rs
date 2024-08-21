use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{payment_method::PaymentMethod, sell_order::Currency};



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSellOrderReq{
    pub amount:BigDecimal,
    pub min_amount:BigDecimal,
    //pub max_amount:BigDecimal, 
    pub currency:Currency,
    pub payment_method: PaymentMethod,
    pub payment_method_id: String,
    pub wallet_address:String,
    pub password:String, 
    pub phone_number:String
}



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateSellOrderReq{
    pub amount: Option<BigDecimal>,
    pub min_amount:Option<BigDecimal>,
    pub max_amount:Option<BigDecimal>, 
    pub currency:Option<Currency>,
    pub payment_method: Option<PaymentMethod>,
    pub payment_method_id: Option<String>
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateBuyOrderReq{
    pub amount:BigDecimal,
    pub sell_order_id: String,
    pub wallet_address:String
}