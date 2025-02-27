

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};
use diesel::prelude::*;
use diesel::associations::Associations;



#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable, AsChangeset)]
#[diesel(belongs_to(User, foreign_key = user_name))]
#[diesel(table_name = crate::schema::profiles)]
pub struct Profile {
    pub id: String,
    pub user_name:String, 
    pub bio: Option<String>,
    pub name:Option<String>,
    pub image:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub app_f_token: Option<String> // app firebase token
}


#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::friends)]
#[diesel(belongs_to(Profile, foreign_key = user_username))]
#[diesel(belongs_to(Profile, foreign_key = friend_username))]
pub struct Friend{
    pub id:i32,
    pub user_username:String,
    pub friend_username:String,
}
