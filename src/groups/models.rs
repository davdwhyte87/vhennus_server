use chrono::NaiveDateTime;
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};

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
        "opensource"
    ]
});
