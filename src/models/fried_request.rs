use std::default;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::profile::Profile;





#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FriendRequest {
    pub id: String,
    pub user_name:String, 
    pub requester: String,
    pub status:String, // 0 pending //1 accepted // 2 rejected , 
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, strum_macros::Display)]
pub enum FriendRequestStatus {
    #[default]
    PENDING,
    ACCEPTED,
    DECLINED
}