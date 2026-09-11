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
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCommentReq {
    pub text: String,
}
