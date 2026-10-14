use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Post {
    pub id: String,
    pub text: String,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Comment {
    pub id: String,
    pub text: String,
    pub user_name: String,
    pub created_at: NaiveDateTime,
    pub post_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Like {
    pub user_name: String,
    pub post_id: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, sqlx::FromRow)]
pub struct PostFeed {
    pub id: String,
    pub image: Option<String>,
    pub text: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_name: String,
    pub name: Option<String>,
    pub profile_image: Option<String>,
    pub like_count: Option<i64>,
    pub comment_count: Option<i64>,
}

#[derive(Serialize, Debug, Deserialize, Clone, sqlx::FromRow)]
pub struct FeedComment {
    pub id: String,
    pub text: String,
    pub user_name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct PostWithComments {
    pub post: PostFeed,
    pub comments: Vec<Comment>,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct PostNotificationTarget {
    pub id: String,
    pub user_name: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePostReq {
    pub text: String,
    #[serde(default)]
    pub image: Option<String>,
    /// Optional poll options attached to the post (2..=5 non-empty strings).
    #[serde(default)]
    pub poll_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCommentReq {
    pub text: String,
}

// ---------------- Polls ----------------

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct PollOption {
    pub id: String,
    pub post_id: String,
    pub text: String,
    pub position: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct PollVote {
    pub id: String,
    pub post_id: String,
    pub option_id: String,
    pub user_name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PollOptionResult {
    pub id: String,
    pub text: String,
    pub position: i32,
    pub vote_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PollData {
    pub options: Vec<PollOptionResult>,
    pub total_votes: i64,
    /// Option id the requesting viewer voted for (if any).
    pub user_voted_option_id: Option<String>,
    pub has_poll: bool,
}

/// PostFeed enriched with its poll (if any). Serialized to clients.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostFeedWithPoll {
    pub id: String,
    pub image: Option<String>,
    pub text: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_name: String,
    pub name: Option<String>,
    pub profile_image: Option<String>,
    pub like_count: Option<i64>,
    pub comment_count: Option<i64>,
    pub poll: Option<PollData>,
}

impl From<PostFeed> for PostFeedWithPoll {
    fn from(p: PostFeed) -> Self {
        Self {
            id: p.id,
            image: p.image,
            text: p.text,
            created_at: p.created_at,
            updated_at: p.updated_at,
            user_name: p.user_name,
            name: p.name,
            profile_image: p.profile_image,
            like_count: p.like_count,
            comment_count: p.comment_count,
            poll: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostWithCommentsAndPoll {
    pub post: PostFeedWithPoll,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct VotePollReq {
    pub option_id: String,
}
