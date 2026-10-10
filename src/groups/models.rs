use std::sync::Arc;
use actix_ws::Session;
use chrono::NaiveDateTime;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use validator::Validate;
use crate::profile::models::MiniProfile;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateGroupReq{
    pub group_id:String,
    pub name:Option<String>,
    pub description:Option<String>,
    pub is_private:Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateRoomReq{
    pub room_id:String,
    pub name:Option<String>,
    pub description:Option<String>,
    pub is_private:Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateGroupReq{
    pub name:String,
    pub description:Option<String>,
    pub image:Option<String>,
    pub is_private:bool,
    pub category:Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRoomReq{
    pub name:String,
    pub group_id:String,
    pub description:Option<String>,
    pub is_private:bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct Group {
    pub id: String,
    pub user_name:String,
    pub name:String,
    pub description:Option<String>,
    pub is_private:bool,
    pub image:Option<String>,
    pub category:Vec<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}
// if group is private it cannot be findable in searches

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct GroupCategory {
    pub id: String,
    pub name:String,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}



#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct Room {
    pub id: String,
    pub group_id:String,
    pub name:String,
    pub description:Option<String>,
    pub is_private:bool,
    pub created_by:String,
    pub code:Option<String>,
    pub member_count:i64,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct RoomView{
    pub id: String,
    pub group_id:String,
    pub name:String,
    pub description:Option<String>,
    pub is_private:bool,
    pub created_by:String,
    pub code:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub member_count:i64
}

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct RoomWithMembersView{
    pub id: String,
    pub group_id:String,
    pub name:String,
    pub description:Option<String>,
    pub is_private:bool,
    pub created_by:String,
    pub code:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub member_count:i64,
    pub members: Vec<MiniProfile>
}

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct MyGroupsView{
    pub id:String,
    pub name:String,
    pub description:Option<String>,
    pub is_private:bool,
    pub created_by:String,
    pub rooms:Vec<Room>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct UserRoom {
    pub user_name: String,
    pub room_id:String,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct RoomRead {
    pub user_name: String,
    pub room_id:String,
    pub last_read:String,
}

// a private room will not show if you are not a part of it

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct RoomMessage {
    pub id:String,
    pub user_name: String,
    pub text:String,
    pub image:Option<String>,
    pub room_id:String,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
}

pub type RoomMembers = Arc<DashMap<String, DashMap<String,()>>>;
pub type UserRoomSessions = Arc<DashMap<String, Session>>;


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum MessageType{
    #[default]
    RoomMessage,
    JoinRoom,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct RoomChatReq{
    pub message_type:MessageType,
    pub room_id:String,
    pub room_name:String,
    pub group_name:String,
    pub message:String,
    pub image:Option<String>,
    pub join_room_code:Option<String>,
}
pub static GROUP_CATEGORIES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "technology",
        "science",
        "art",
        "music",
        "sports",
        "health",
        "education",
        "gaming",
        "finance",
        "travel",
        "news",
        "lifestyle",
        "programming",
        "movies",
        "books",
        "fashion",
        "food",
        "fitness",
        "photography",
        "history",
        "culture",
        "relationships",
        "parenting",
        "business",
        "entrepreneurship",
        "marketing",
        "self-improvement",
        "mental-health",
        "memes",
        "crypto",
        "blockchain",
        "nfts",
        "design",
        "productivity",
        "spirituality",
        "philosophy",
        "politics",
        "career",
        "environment",
        "animals",
        "nature",
        "events",
        "cars",
        "space",
        "diy",
        "architecture",
        "languages",
        "coding",
        "android",
        "ios",
        "web development",
        "ai",
        "ml",
        "data science",
        "devops",
        "security",
        "opensource",
    ]
});

// =====================================================================
// Groups v2: one group = one chat feed (no rooms).
// Legacy Room* structs above are frozen; new code uses the types below.
// All DB rows use runtime-checked `sqlx::query_as::<_, T>` + FromRow
// so compilation does not need a live DATABASE_URL.
// =====================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateGroupV2Req {
    pub name: String,
    pub about: Option<String>,
    pub image: Option<String>,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdateGroupV2Req {
    pub name: Option<String>,
    pub about: Option<String>,
    pub image: Option<String>,
    pub is_private: Option<bool>,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct DbGroup {
    pub id: String,
    pub user_name: String,
    pub name: String,
    pub about: Option<String>,
    pub description: Option<String>,
    pub is_private: bool,
    pub image: Option<String>,
    pub invite_code: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct GroupCategoryRow {
    pub id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct GroupMemberRow {
    pub group_id: String,
    pub user_name: String,
    pub role: String,
    pub last_read_at: NaiveDateTime,
    pub last_read_msg_id: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct GroupTopic {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub is_open: bool,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct GroupMessage {
    pub id: String,
    pub group_id: String,
    pub sender: String,
    pub text: String,
    pub image: Option<String>,
    pub topic_id: Option<String>,
    pub reply_to_msg_id: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GroupReplyPreview {
    pub id: String,
    pub sender: String,
    pub sender_name: Option<String>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GroupMessageView {
    pub id: String,
    pub group_id: String,
    pub sender: String,
    pub sender_name: Option<String>,
    pub sender_image: Option<String>,
    pub text: String,
    pub image: Option<String>,
    pub topic_id: Option<String>,
    pub reply_to: Option<GroupReplyPreview>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct GroupJoinRequest {
    pub id: String,
    pub group_id: String,
    pub user_name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct JoinRequestView {
    pub id: String,
    pub group_id: String,
    pub user_name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub profile: Option<MiniProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GroupDetail {
    pub id: String,
    pub name: String,
    pub about: Option<String>,
    pub image: Option<String>,
    pub is_private: bool,
    pub owner: String,
    pub invite_code: Option<String>,
    pub categories: Vec<String>,
    pub member_count: i64,
    pub is_member: bool,
    pub my_role: Option<String>,
    pub open_topic: Option<GroupTopic>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MyGroupItem {
    pub id: String,
    pub name: String,
    pub about: Option<String>,
    pub image: Option<String>,
    pub is_private: bool,
    pub owner: String,
    pub categories: Vec<String>,
    pub member_count: i64,
    pub unread_count: i64,
    pub my_role: Option<String>,
    pub open_topic: Option<GroupTopic>,
    pub last_message: Option<GroupMessageView>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTopicReq {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkReadReq {
    pub last_msg_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespondJoinReq {
    /// "accept" | "reject"
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminCategoryReq {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupUnreadItem {
    pub group_id: String,
    pub unread: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GroupUnreadResp {
    pub total: i64,
    pub groups: Vec<GroupUnreadItem>,
}

/// Multi-device WS state (compare with single-session UserRoomSessions).
/// sessions: user_name -> (conn_id -> Session)
/// presence: group_id -> set of online user_names
pub type GroupSessions = Arc<DashMap<String, DashMap<String, Session>>>;
pub type GroupPresence = Arc<DashMap<String, DashMap<String, ()>>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GroupClientFrame {
    Join { group_id: String },
    Send {
        temp_id: Option<String>,
        group_id: String,
        message: Option<String>,
        image: Option<String>,
        #[serde(default)]
        reply_to_msg_id: Option<String>,
    },
    Read { group_id: String, msg_id: Option<String> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GroupServerFrame {
    Joined { groups: Vec<String> },
    Sent { temp_id: Option<String>, message: GroupMessageView },
    New { message: GroupMessageView },
    Error { temp_id: Option<String>, message: String },
    Unread { total: i64, groups: Vec<GroupUnreadItem> },
    Presence { group_id: String, online: Vec<String> },
}

pub fn invite_code() -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    thread_rng().sample_iter(&Alphanumeric).take(12).map(char::from).collect()
}

pub fn slug_category(name: &str) -> String {
    format!(
        "cat-{}",
        name.trim().to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-")
    )
    .replace("--", "-")
}
