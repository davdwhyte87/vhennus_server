use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub user_name: String,
    pub email: Option<String>,
    pub code: Option<i32>,
    pub created_at: NaiveDateTime,
    pub user_type: String, // 0 user, 1 admin
    pub password_hash: String,
    pub is_deleted: bool,
    pub email_confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct FriendRequest {
    pub id: String,
    pub user_name:String, 
    pub requester: String,
    pub status:String, // 0 pending //1 accepted // 2 rejected , 
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, strum_macros::Display)]
pub enum FriendRequestStatus {
    #[default]
    PENDING,
    ACCEPTED,
    DECLINED
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Friend{
    pub id:i32,
    pub user_username:String,
    pub friend_username:String,
}

#[derive(Default, Serialize, Debug, Deserialize, Clone, sqlx::FromRow)]
pub struct FriendRequestWithProfile{
    pub id: String,
    pub user_name: String,
    pub requester:String,
    pub status:String,
    pub created_at: NaiveDateTime,
    pub bio:Option<String>,
    pub name:Option<String>,
    pub image:Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReq{
    pub user_name:String,
    pub password:String,
    pub user_type:String,
    pub email:String,
    pub referral:Option<String>
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct  SendFriendReq{
    pub user_name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct  LoginReq{
    pub user_name:String,
    pub password:String
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct  ConfirmAccountReq{
    pub code:String,
    pub email:String
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct  ResendCodeReq{
    pub email:String
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct  GetCodeReq{
    #[validate(email)]
    pub email:String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct  CreateKuracoinID{
    #[validate(length(min=1))]
    pub user_name:String,
    pub password:String
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordReq{
    pub code:String,
    pub password:String,
    pub user_name:String
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetPasswordResetCodeReq{
    pub user_name:String,
}