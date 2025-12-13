

use serde_derive::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderMessageReq{
    pub receiver_user_name:String,
    pub text:String,
    pub image:String, 
    pub buy_order_id:String
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileReq{
    pub bio:Option<String>, 
    pub image:Option<String>, 
    pub name:Option<String>,
    pub app_f_token:Option<String>,
    pub new_earning:Option<String>,
    pub new_referrals:Option<Vec<String>>,
    pub earnings_wallet:Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateGroupReq{
    pub group_id:String,
    pub name:Option<String>,
    pub description:Option<String>,
    pub is_private:Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateRoomReq{
    pub room_id:String,
    pub name:Option<String>,
    pub description:Option<String>,
    pub is_private:Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGroupReq{
    pub name:String,
    pub description:Option<String>,
    pub image:Option<String>,
    pub is_private:bool,
    pub category:Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRoomReq{
    pub name:String,
    pub group_id:String,
    pub description:Option<String>,
    pub is_private:bool,
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AddWallet{
    pub address:String,
    pub message:String,
    pub signature:String,
}



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetAllOrderMessageReq{
    pub buy_order_id:String
}
