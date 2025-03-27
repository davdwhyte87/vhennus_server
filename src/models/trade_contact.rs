use serde::{Deserialize, Serialize};
use std::{default, string::ToString};
use strum_macros;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeContact {
    pub id:String,
    pub phone_number:String
}