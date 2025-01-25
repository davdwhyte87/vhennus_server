use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_name: String,
    pub email:Option<String>,
    pub code:Option<i32>,
    pub created_at:String,
    pub user_type:UserType,
    pub password_hash:String,
    pub is_deleted: bool
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserType{
    User,
    Admin
}