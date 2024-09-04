use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentMethodData {
    pub id:String,
    pub name:String,
    pub user_name:String,
    pub payment_method: PaymentMethod,
    pub account_name:String,
    pub account_number:String,
    pub bank_name:String,
    pub other:String,
    pub paypal_email:String, 
    pub venmo_username:String, 
    pub skrill_email:String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaypalPaymentMethod {
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,strum_macros::Display, Default)]
pub enum PaymentMethod {
    #[default]
    Bank,
    Paypal,
    Skrill,
    Cash
}

