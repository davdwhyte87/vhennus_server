use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RefClick {
    pub click_id: String,
    pub code:String,
    pub created_at: NaiveDateTime,
}