

use chrono::format::Numeric;
use bigdecimal::BigDecimal;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use diesel::{AsExpression, FromSqlRow, Insertable, Queryable};
use strum_macros;
use diesel::prelude::*;
use super::{buy_order::BuyOrder, comment::Comment, payment_method::{PaymentMethod, PaymentMethodData}, user::UserType};




#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::system_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct System {
    pub id: i32,
    pub price: BigDecimal,
    pub android_app_version:String,
    pub trivia_win_amount:BigDecimal
}
