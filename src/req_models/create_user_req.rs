
use serde_derive::{Deserialize, Serialize};
use crate::models::user::UserType;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReq{
    pub user_name:String,
    pub password:String,
    pub user_type:UserType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReq{
    pub user_name:String,
    pub password:String,
}