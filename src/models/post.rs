

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Post {
    pub id: String,
    pub text:String, 
    pub image: String, 
    pub created_at:String,
    pub user_name:String, 
    pub number_of_likes:i32, 
    pub comments_ids: Vec<String>,
    pub comments: Option<Vec<Comment>>,
    pub number_of_views:i32
}


