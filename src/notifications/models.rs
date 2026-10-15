use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Stored inbox row. `kind` is one of:
/// comment | like | poll_vote | friend_request | friend_accepted
#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct NotificationItem {
    pub id: String,
    pub user_name: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub is_read: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MarkReadReq {
    /// If omitted/empty, all of the viewer's notifications are marked read.
    pub ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterTokenReq {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UnreadCountResp {
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BroadcastReq {
    #[validate(length(min = 1, max = 140))]
    pub title: String,
    #[validate(length(min = 1, max = 500))]
    pub body: String,
    pub link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BroadcastResp {
    pub users_notified: i64,
    pub pushes_sent: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PushStatusResp {
    pub total_users: i64,
    pub users_with_tokens: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TestPushReq {
    pub user_name: String,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TestPushResp {
    pub stored: bool,
    pub has_token: bool,
    pub push_sent: bool,
}
