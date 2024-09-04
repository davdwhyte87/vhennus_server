

use serde_derive::{Deserialize, Serialize};
use validator::Validate;
use crate::models::user::UserType;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderMessageReq{
    pub receiver_user_name:String,
    pub text:String,
    pub image:String, 
    pub buy_order_id:String
}



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetAllOrderMessageReq{
    pub buy_order_id:String
}
