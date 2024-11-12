use std::default;

use serde::{Deserialize, Serialize};




#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FriendRequest {
    pub id: String,
    pub user_name:String, 
    pub requester: String,
    pub status:FriendRequestStatus,
    pub created_at:String,
    pub updated_at:String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, strum_macros::Display)]
pub enum FriendRequestStatus {
    #[default]
    PENDING,
    ACCEPTED,
    DECLINED
}