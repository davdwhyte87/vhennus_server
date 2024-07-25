

use mongodb::bson::oid::ObjectId;
use serde_derive::{Deserialize, Serialize};


// this is a model for a patients tests data which belongs to a test record 
// test data is information on a single medical test, for example, a typhoid test
#[derive(Debug, Serialize, Deserialize)]
pub struct TestData {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub nurse_email:String,
    pub created_at: String,
    pub updated_at: String,
    pub test_record_id: ObjectId,
    pub name: String,
    pub result:String,
}
