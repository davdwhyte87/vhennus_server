use serde::{Deserialize, Serialize};
use std::string::ToString;
use strum_macros;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankPaymentMethod {
    pub id:String,
    pub account_name:String,
    pub account_number:String,
    pub bank_name:String,
    pub other:String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaypalPaymentMethod {
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,strum_macros::Display )]
pub enum PaymentMethod {
    Bank,
    Paypal,
    Skrill,
    Cash
}

