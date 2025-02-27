use std::default;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use super::profile::Profile;





#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::friend_requests)]
#[diesel(belongs_to(Profile,foreign_key=requester))]
#[diesel(belongs_to(Profile,foreign_key=user_name))]
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