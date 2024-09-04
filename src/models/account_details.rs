

use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};


// player account details
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountDetails {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub account_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub account_number : String,
    pub bank_name:String,
    pub user_email:String

}