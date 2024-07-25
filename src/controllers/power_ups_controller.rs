use std::borrow::{Borrow, BorrowMut};
use actix_web::{get, HttpResponse, post, put};
use actix_web::web::{Data, Json, Path, ReqData};
use mongodb::bson::oid::ObjectId;
use validator::Validate;
use crate::models::power_up::{get_enum_string, get_price, PlayerPowerUp, PowerUpType};

use crate::models::request_models::{BuyPowerUpReq, UpdateTestDataReq, UsePowerUpReq};
use crate::models::response::{GetPowerupsResp, Response, ResponsePlayerPowerUp};
use crate::models::test_data::TestData;
use crate::models::test_record::TestRecord;
use crate::models::user::UserType;
use crate::req_models::create_test_data_req::CreateTestDataReq;
use crate::services::mongo_service::MongoService;
use crate::services::power_up_service::PowerUpService;
use crate::services::wallet_service::WalletService;
use crate::utils::auth::Claims;


// when a player users a power up in the game app, it sends a request to the server,
// telling it that the player has used this power up.
#[post("power_up/use")]
pub async fn use_power_up(database:Data<MongoService>,
                              req_data :Json<UsePowerUpReq>,
                              claim:Option<ReqData<Claims>>
)
                              ->HttpResponse
{

    // validate request
    match req_data.borrow().validate() {
        Ok(_) => {},
        Err(err) => {
            return HttpResponse::BadRequest().json(err);
        }
    }

    // get claim data
    let claim = match claim {
        Some(claim) => { claim },
        None => {
            return HttpResponse::Unauthorized()
                .json(Response { message: "Not authorized".to_string() })
        }
    };

    // get specific power up for the user
    let player_powerup =PowerUpService::get_user_power_up(
        &database.db,
        claim.email.to_string(),
        get_enum_string(&req_data.power_up_type)
    ).await;
    let mut player_powerup = match player_powerup {
        Ok(player_powerup)=>{player_powerup},
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    };

    // update power up
    // check if there are no power ups
    if player_powerup.amount == 0 || player_powerup.amount<0{
        return HttpResponse::BadRequest()
            .json(Response{message:"Not enough powerup".to_string()})
    }
    player_powerup.amount = player_powerup.amount -1;
    let update_res= PowerUpService::update(
        &database.db,
        player_powerup.user_email.to_owned(),
        get_enum_string(&player_powerup.power_up_type),
        &player_powerup
    ).await;
    match update_res {
        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})

        }
    }
    return  HttpResponse::Ok().json(
        ResponsePlayerPowerUp{power_up:player_powerup}
    );
}


#[post("power_up/buy")]
pub async fn buy_power_up(database:Data<MongoService>,
                          req_data :Json<BuyPowerUpReq>,
                          claim:Option<ReqData<Claims>>
) ->HttpResponse
{

    // validate request
    match req_data.validate() {
        Ok(_) => {},
        Err(err) => {
            return HttpResponse::BadRequest().json(err);
        }
    }

    // get claim data
    let claim = match claim {
        Some(claim) => { claim },
        None => {
            return HttpResponse::Unauthorized()
                .json(Response { message: "Not authorized".to_string() })
        }
    };

    // get the total amount of stuff player wants to buy
    let total_amount = &req_data.amount * get_price(&req_data.power_up_type);
    // println!("total amount {}", total_amount);
    // get player wallet
    let player_email = &claim.email;
    let wallet =WalletService::get_by_email(
        &database.db,&player_email.to_string()
    ).await;
    let mut wallet = match wallet {
        Ok(wallet)=>{
            match wallet {
                Some(wallet)=>{wallet},
                None=>{return HttpResponse::NotFound()
                    .json(Response{message:"No wallet found".to_string()})}
            }
        },
        Err(err)=>{
            return HttpResponse::BadRequest().json(Response{message:err.to_string()});
        }
    };
    //check if the player has the cash
    if total_amount>wallet.amount{
        return HttpResponse::BadRequest().json(Response{message:"Insufficient fund".to_string()});
    }
    wallet.amount = wallet.amount - total_amount;

    // update wallet
    let wallet_update_res = WalletService::update(
        &database.db,
        wallet.id.unwrap().to_string(),
        &wallet ).await;
    match wallet_update_res {
        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::InternalServerError().json(Response{message:err.to_string()});
        }
    }
    // get player power up data
    let player_powerup =PowerUpService::get_user_power_up(&database.db,player_email.to_string(),
                                                          get_enum_string(&req_data.power_up_type)).await;
    let mut player_powerup = match player_powerup {
        Ok(player_powerup)=>{player_powerup},
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    };
    player_powerup.amount = req_data.amount;
   //update
    let player_poweup_update_res = PowerUpService::update(
        &database.db, player_email.to_string(),
        get_enum_string(&player_powerup.power_up_type),
        &player_powerup
    ).await;
    match player_poweup_update_res {
        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    }
    // TODO: Use transaction function to update database
    return  HttpResponse::Ok().json(
        Response{message:"Power up purchased successfully".to_string()}
    );
}


#[get("power_up/get_all")]
pub async fn get_player_powerups(database:Data<MongoService>,
                          claim:Option<ReqData<Claims>>
) ->HttpResponse
{


    // get claim data
    let claim = match claim {
        Some(claim) => { claim },
        None => {
            return HttpResponse::Unauthorized()
                .json(Response { message: "Not authorized".to_string() })
        }
    };

    let player_powerups = PowerUpService::get_player_power_ups(
        &database.db,
        &claim.email
    ).await;
    let player_powerups =match player_powerups {
        Ok(player_powerups)=>{player_powerups},
        Err(err)=>{
            println!(" errpr : {}",err.to_string());
            return HttpResponse::InternalServerError()
            .json(Response{message:err.to_string()})
        }
    };
    return HttpResponse::Ok().json(GetPowerupsResp{
        power_ups:player_powerups
    })
}