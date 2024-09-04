

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{payment_method::PaymentMethod, sell_order::Currency};



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePaymentMethodReq{
    pub payment_method: PaymentMethod,
    pub account_name:String,
    pub account_number:String,
    pub bank_name:String,
    pub other:String,
    pub paypal_email:String, 
    pub venmo_username:String, 
    pub skrill_email:String,
    pub name:String
}
