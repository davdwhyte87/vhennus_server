

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
    pub app_f_token:Option<String>
}



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetAllOrderMessageReq{
    pub buy_order_id:String
}
