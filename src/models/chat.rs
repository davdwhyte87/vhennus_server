

use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};


#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::chats)]
pub struct Chat {
    pub id: String,
    pub sender:String,
    pub receiver:String,
    pub message:String,
    pub image:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub pair_id:String
}
