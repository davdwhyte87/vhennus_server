use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;

use super::{buy_order::BuyOrder, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriviaQuestion {
    pub id: String,
    pub question: String,
    pub options: Vec<String>, 
    pub answer: String,
    pub is_used: bool
}

// the logic is that we get all the questions in the database and the first 
// one to have is_used as false, we return as the todays question