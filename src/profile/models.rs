use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Default, sqlx::FromRow)]
pub struct Profile {
    pub id: String,
    pub user_name:String, 
    pub bio: Option<String>,
    pub name:Option<String>,
    pub image:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub app_f_token: Option<String> ,// app firebase token
    pub wallets: Option<String>,
    pub unclaimed_earnings:BigDecimal,
    pub is_earnings_activated:bool,
    pub referred_users: Vec<String>,
    pub earnings_wallet: Option<String>
}


#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MiniProfile{
    pub user_name:String,
    pub image: Option<String>,
    pub bio:Option<String>,
    pub name: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProfileWithFriends{
    pub profile: Profile,
    pub friends: Vec<MiniProfile>
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileReq{
    pub bio:Option<String>, 
    pub image:Option<String>, 
    pub name:Option<String>,
    pub app_f_token:Option<String>,
    pub new_earning:Option<String>,
    pub new_referrals:Option<Vec<String>>,
    pub earnings_wallet:Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AddWallet{
    pub address:String,
    pub message:String,
    pub signature:String,
}