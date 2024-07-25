use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::sell_order::Currency;



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSellOrderReq{
    pub amount:BigDecimal,
    pub min_amount:BigDecimal,
    pub max_amount:BigDecimal, 
    pub currency:Currency
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateBuyOrderReq{
    pub amount:BigDecimal,
    pub sell_order_id: String
}