
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Like{
    pub user_name:String,
    pub post_id:String,
}