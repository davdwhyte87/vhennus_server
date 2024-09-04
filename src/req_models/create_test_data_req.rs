
use serde_derive::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTestDataReq{
    pub name:String,
    pub result:String,
    pub test_record_id:String,
    #[validate(email)]
    pub nurse_email:String
}