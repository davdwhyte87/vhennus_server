use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LiveRateResponse {
    pub success:bool,
    pub terms:String,
    pub privacy:String,
    pub timestamp: i64,
    pub source:String,
    pub quotes:HashMap<String, f64>
}