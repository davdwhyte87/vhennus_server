use std::borrow::{Borrow, BorrowMut};
use actix_web::{get, HttpResponse, post, put};
use actix_web::web::{Data, Json, Path, ReqData};
use mongodb::bson::oid::ObjectId;
use mongodb::Client;
use serde_json::json;
use validator::Validate;
use crate::models::account_details::AccountDetails;
use crate::models::power_up::{get_price, PlayerPowerUp, PowerUpType};

use crate::models::request_models::{BuyPowerUpReq, CreateAccountDetailsReq, UpdatePlayerRunReq, UpdateTestDataReq, UsePowerUpReq};
use crate::models::response::{PlayerRunInfoRes, Response, ResponsePlayerPowerUp};
use crate::models::test_data::TestData;
use crate::models::test_record::TestRecord;
use crate::models::user::UserType;
use crate::req_models::create_test_data_req::CreateTestDataReq;
use crate::services::account_service::AccountDetailsService;
use crate::services::mongo_service::MongoService;
use crate::services::power_up_service::PowerUpService;
use crate::services::run_info_service::RunInfoService;
use crate::services::wallet_service::WalletService;
use crate::utils::auth::Claims;


#[post("player/update_stats")]
pub async fn update_player_stats(database:Data<MongoService>,
                          req_data :Json<UpdatePlayerRunReq>,
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
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    };

    // get player stats
    let run_info = RunInfoService::get_user_run_info(
        &database.db,
        claim.email.clone()
    ).await;
    let mut run_info = match run_info {
        Ok(run_info)=>{run_info},
        Err(err)=>{return HttpResponse::InternalServerError()
            .json(Response{message:err.to_string()})}
    };

    // update stats
    run_info.distance = req_data.distance;
    // update high score
    if run_info.distance > run_info.high_score{
        run_info.high_score = run_info.distance
    }

    // update stats
    let update_run_res = RunInfoService::update_player_run_info(
        &database.db,
        claim.email.clone(),
        &run_info
    ).await;
    match update_run_res {
        Ok(update_run_res)=>{
            return HttpResponse::Ok().json(
                PlayerRunInfoRes{run_info:run_info}
            )
        },
        Err(err)=>{
            return HttpResponse::BadRequest().json(
                Response{message:err.to_string()}
            )
        }
    }

    // return  HttpResponse::Ok().json(
    //     Response{message:"Power up purchased successfully".to_string()}
    // );
}


#[get("player/get_stats")]
pub async fn get_player_stats(database:Data<MongoService>,
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

    // get run info data
    let run_info = RunInfoService::get_user_run_info(
        &database.db,
        claim.email.clone()
    ).await;
    let mut run_info = match run_info {
        Ok(run_info)=>{run_info},
        Err(err)=>{return HttpResponse::InternalServerError()
            .json(Response{message:err.to_string()})}
    };

    return  HttpResponse::Ok().json(
        PlayerRunInfoRes{run_info:run_info}
    );
}

// create player account

#[post("player/add_account_details")]
pub async fn add_account_details(database:Data<MongoService>,
                              req_data :Json<CreateAccountDetailsReq>,
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

    // get the claims
    // get claim data
    let claim = match claim {
        Some(claim) => { claim },
        None => {
            return HttpResponse::Unauthorized()
                .json(Response { message: "Not authorized".to_string() })
        }
    };
    //check if the user has account details
    let account_details =AccountDetailsService::get_by_email(&database.db, claim.email.to_string()).await;
    let account_details = match account_details {
        Ok(account_details)=>{account_details},
        Err(err)=>{return HttpResponse::InternalServerError()
            .json(Response{message:err.to_string()})}
    };
    let mut should_create = false;
    let mut should_update = false;
    let mut account_details = match account_details {
        Some(account_details)=>{
            // update account details
            should_update =true;
            account_details
        },
        None=>{
            // create account details
            should_create= true;
            let account_details = AccountDetails{
                id:None,
                user_email:claim.email.to_string(),
                account_number: req_data.account_number.to_string(),
                account_name: req_data.account_name.to_string(),
                bank_name:req_data.bank_name.to_owned(),
                created_at:chrono::offset::Utc::now().to_string(),
                updated_at:chrono::offset::Utc::now().to_string()
            };
            account_details
        }
    };

    if should_update {
        account_details.updated_at = chrono::offset::Utc::now().to_string();
        if req_data.account_name !="" {
            account_details.account_name = req_data.account_name.to_owned()
        }
        if req_data.account_number !="" {
            account_details.account_number = req_data.account_number.to_owned()
        }
        if req_data.bank_name !="" {
            account_details.bank_name = req_data.bank_name.to_owned()
        }
        let update_res = AccountDetailsService::update(&database.db, &claim.email, &account_details).await;
        match update_res {
            Ok(_)=>{},
            Err(err)=>{
                return HttpResponse::InternalServerError()
                    .json(Response{message:err.to_string()})
            }
        }
    }
    if should_create{
        let create_res = AccountDetailsService::create(
            &database.db, &account_details
        ).await;
        match create_res {
            Ok(_)=>{},
            Err(err)=>{
                return HttpResponse::InternalServerError()
                    .json(Response{message:err.to_string()})
            }
        }
    }
    return HttpResponse::Ok().json(Response{message:"OK".to_string()})

}
