use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Chat {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub message: String,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub pair_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct ChatPair {
    pub id: String,
    pub user1: String,
    pub user2: String,
    pub last_message: Option<String>,
    pub all_read: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct ChatPairView {
    pub id: String,
    pub user1: String,
    pub user2: String,
    pub last_message: Option<String>,
    pub all_read: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user1_image: Option<String>,
    pub user2_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Circle {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub owner: String,
    pub members: Vec<String>,
    pub image: String,
    pub is_private: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetChatsView {
    pub chat_pair: ChatPairView,
    pub chats: Vec<Chat>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateChatReq {
    pub pair_id: Option<String>,

    pub receiver: String,

    pub message: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateChatPairReq {
    pub user_name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGroupChatReq {
    pub name: String,
    pub display_name: String,
    pub members: Vec<String>,
    pub image: String,
}
