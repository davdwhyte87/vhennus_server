


use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::{self, ToString}};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use strum_macros;

use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, profile::Profile, user::UserType};



#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::chat_pairs)]
pub struct ChatPair {
    pub id: String,
    pub user1:String,
    pub user2:String,
    pub last_message:Option<String>,
    pub all_read: bool,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}
