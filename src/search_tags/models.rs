use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub const TAG_KIND_SKILL: &str = "skill";
pub const TAG_KIND_INTEREST: &str = "interest";

/// Caps keep profiles and result cards readable no matter how many tags exist.
pub const MAX_SKILLS_PER_USER: usize = 5;
pub const MAX_INTERESTS_PER_USER: usize = 10;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SearchTag {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTagReq {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTagReq {
    pub name: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchFriendsReq {
    pub query: Option<String>,
    pub skill_ids: Option<Vec<String>>,
    pub interest_ids: Option<Vec<String>>,
    pub limit: Option<i64>,
}

/// One search hit: profile basics + attached tags split by kind.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendSearchHit {
    pub user_name: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub skills: Vec<SearchTag>,
    pub interests: Vec<SearchTag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetMyTagsReq {
    pub tag_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyTagsResp {
    pub tag_ids: Vec<String>,
    pub tags: Vec<SearchTag>,
}

pub fn normalize_kind(raw: &str) -> Option<String> {
    let lower = raw.trim().to_lowercase();
    match lower.as_str() {
        "skill" | "skills" => Some(TAG_KIND_SKILL.to_string()),
        "interest" | "interests" => Some(TAG_KIND_INTEREST.to_string()),
        _ => None,
    }
}

pub fn normalize_name(raw: &str) -> String {
    raw.trim().to_lowercase()
}
