
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};


// this represents a power up in the video game.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerPowerUp {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_email: String,
    pub created_at: String,
    pub amount : i32,
    pub power_up_type:PowerUpType,
    pub in_game_amount: i32,
}

impl PlayerPowerUp{
  
}

pub fn get_price(power_up_type:&PowerUpType) ->i32{
    match power_up_type {
        PowerUpType::Phasing => {return 10}
        PowerUpType::Blast => {return 20}
        PowerUpType::SlowMotion => {return 30}
    }

    return 0
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PowerUp {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub created_at: String,
    pub price : u64,
    pub power_up_type:PowerUpType,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PowerUpType {
    Phasing,
    Blast,
    SlowMotion
}

pub fn get_enum_string(power_up:&PowerUpType)->String{
    match power_up {
        PowerUpType::Phasing=>{"Phasing".to_string()},
        PowerUpType::Blast=>{"Blast".to_string()},
        PowerUpType::SlowMotion=>{"SlowMotion".to_string()},
    }
}