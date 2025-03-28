
use serde_derive::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserReq{
    pub user_name:String,
    pub password:String,
    pub user_type:String,
    pub email:String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReq{
    pub user_name:String,
    pub password:String,
}