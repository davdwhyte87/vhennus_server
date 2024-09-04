use actix_web::{get, HttpResponse, post};
use actix_web::web::{Data, Json, ReqData};
use validator::Validate;
use crate::models::request_models::BuyCoinReq;
use crate::models::response::{GetWalletResp, Response};
use crate::services::mongo_service::MongoService;
use crate::services::wallet_service::WalletService;
use crate::utils::auth::Claims;

#[post("wallet/buy_coin")]
pub async fn buy_coin(database:Data<MongoService>,
                                 req_data :Json<BuyCoinReq>,
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

    // get wallet data
    let wallet = WalletService::get_by_email(&database.db, &claim.email ).await;
    let mut wallet = match wallet {
        Ok(wallet)=> {
            match wallet {
                Some(wallet)=>{wallet},
                None=>{return HttpResponse::NotFound()
                    .json(Response{message:"No wallet found".to_string()})}
            }
        },
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    };

    // check payment processor
    // update wallet amount
    wallet.amount = 10;
    // update wallet
    let update_ok = WalletService::update(
        &database.db,
        wallet.id.unwrap().to_string(),
        &wallet
    ).await;
    match update_ok {
        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    }

    //
    return HttpResponse::Ok().json(Response{message:"OK".to_string()})
}


#[get("wallet/get_wallet")]
pub async fn get_wallet(database:Data<MongoService>,
                      claim:Option<ReqData<Claims>>
) ->HttpResponse
{
    

    // get claim data
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    };

    // get wallet data
    let wallet = WalletService::get_by_email(&database.db, &claim.email ).await;
    let wallet = match wallet {
        Ok(wallet)=> {
            match wallet {
                Some(wallet)=>{wallet},
                None=>{return HttpResponse::NotFound()
                    .json(Response{message:"No wallet found".to_string()})}
            }
        },
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    };

    return HttpResponse::Ok().json(GetWalletResp{wallet:wallet})
}

