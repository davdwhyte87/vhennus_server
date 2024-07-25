

use mongodb::bson::oid::ObjectId;
use serde_derive::{Deserialize, Serialize};
use crate::models::test_data::TestData;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub nurse_email:String,
    pub created_at: String,
    pub diagnosis_id: Option<String>,
    pub note: String,
    pub patient_email:String,
    pub test_datas: Vec<ObjectId>,
    pub test_data:Option<Vec<TestData>>
}
