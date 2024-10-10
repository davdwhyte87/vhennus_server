use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;

use super::{buy_order::BuyOrder, payment_method::{PaymentMethod, PaymentMethodData}, trivia_question::TriviaQuestion, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriviaGame {
    pub id: String,
    pub trivia_question_id:String,
    pub winner_user_name: Option<String>,
    pub date:String,
    pub is_ended:bool,
    pub trivia_question:Option<TriviaQuestion>
}
