

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, profile::Profile, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Post {
    pub id: String,
    pub text:String, 
    pub image: String, 
    pub created_at:String,
    pub user_name:String, 
    pub likes:Vec<String>, //usernames of people who liked 
    pub comments_ids: Vec<String>,
    pub comments: Option<Vec<Comment>>,
    pub number_of_views:i32,
    pub profile:Option<Profile>
}


