use mongodb::results::InsertOneResult;
use serde::Serialize;
use serde_derive::Deserialize;
use crate::models::power_up::{PlayerPowerUp, PowerUp};
use crate::models::run_info::RunInfo;
use crate::models::wallet::Wallet;


#[derive(Serialize)]
pub struct Response {
    pub message: String,

}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BResponse<T> {
    pub status: i32,
    pub message: String,
    pub data: Option<T>,
}



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericResp<T> {
    pub message: String,
    pub server_message: Option<String>,
    pub data:Option<T>
}





#[derive(Serialize)]
pub struct ResponseInsert{
    pub message: String,
    pub data: InsertOneResult
}

#[derive(Serialize)]
pub struct LoginResp{
    pub message: String,
    pub token: String
}


#[derive(Serialize)]
pub struct CodeResp{
    pub code: i32
}

#[derive(Serialize)]
pub struct ResponsePlayerPowerUp {
    pub power_up: PlayerPowerUp,
}

#[derive(Serialize)]
pub struct PlayerRunInfoRes {
    pub run_info: RunInfo,
}

#[derive(Serialize)]
pub struct GetWalletResp {
    pub wallet: Wallet,
}

#[derive(Serialize)]
pub struct GetPowerupsResp {
    pub power_ups: Vec<PlayerPowerUp>,
}

