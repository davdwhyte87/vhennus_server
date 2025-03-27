

use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};


// every player has run info
#[derive(Debug, Serialize, Deserialize)]
pub struct RunInfo {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_email: String,
    pub created_at: String,
    pub updated_at: String,
    pub distance : i32,
    pub high_score:i32,
    
}