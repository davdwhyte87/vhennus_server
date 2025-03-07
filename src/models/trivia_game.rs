use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;
use crate::models::trivia_question::TriviaQuestion;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriviaGame {
    #[serde(rename = "_id")]
    pub mongo_id: ObjectId,
    pub id: String,
    pub trivia_question_id:String,
    pub winner_user_name: Option<String>,
    pub date:String,
    pub is_ended:bool,
    pub trivia_question:Option<TriviaQuestion>
}
