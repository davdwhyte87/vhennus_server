use std::default;

use serde::{Deserialize, Serialize};

use super::profile::Profile;




#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FriendRequest {
    pub id: String,
    pub user_name:String, 
    pub requester: String,
    pub status:FriendRequestStatus,
    pub created_at:String,
    pub updated_at:String,
    pub requester_profile: Option<Profile>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, strum_macros::Display)]
pub enum FriendRequestStatus {
    #[default]
    PENDING,
    ACCEPTED,
    DECLINED
}