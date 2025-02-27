

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use strum_macros;

use super::{buy_order::BuyOrder, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::comments)]
pub struct Comment {
    pub id:String, 
    pub text:String,
    pub user_name:String,
    pub created_at:NaiveDateTime,
    pub post_id:String
}