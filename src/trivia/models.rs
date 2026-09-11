use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriviaQuestion {
    #[serde(rename = "_id")]
    pub mongo_id: ObjectId,
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
    pub is_used: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriviaGame {
    #[serde(rename = "_id")]
    pub mongo_id: ObjectId,
    pub id: String,
    pub trivia_question_id: String,
    pub winner_user_name: Option<String>,
    pub date: String,
    pub is_ended: bool,
    pub trivia_question: Option<TriviaQuestion>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AnswerGame {
    pub answer: String,
    pub wallet_address: String,
}
