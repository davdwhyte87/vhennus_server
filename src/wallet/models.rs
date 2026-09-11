use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_email: String,
    pub created_at: String,
    pub amount: i32,
}

#[derive(Serialize)]
pub struct GetWalletResp {
    pub wallet: Wallet,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PowerUpType {
    Phasing,
    Blast,
    SlowMotion
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BuyCoinReq {
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UsePowerUpReq {
    pub power_up_type: PowerUpType,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BuyPowerUpReq {
    pub power_up_type: PowerUpType,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePlayerRunReq {
    pub distance: i32,
}
