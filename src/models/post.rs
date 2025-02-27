

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, profile::Profile, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::posts)]
pub struct Post {
    pub id: String,
    pub text:String, 
    pub image: Option<String>, 
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub user_name:String,
}


