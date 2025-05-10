use std::env;
use std::io::BufRead;
use std::str::FromStr;
use actix_web::{get, post, web::{self, Data, ReqData}, HttpResponse, ResponseError};
use bigdecimal::{BigDecimal, Zero};
use chrono::Local;
use log::{debug, error};
use serde::Deserialize;
use serde_json::from_str;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{models::{profile::Profile, response::GenericResp}, req_models::{create_sell_order_req::CreatePostReq, requests::UpdateProfileReq}, services::{mongo_service::MongoService, profile_service::ProfileService}, utils::auth::Claims, CONFIG};
use crate::controllers::user_controller::resend_code;
use crate::models::request_models::{BRequest, BTransfer, BVerifyWallet, TransferReq};
use crate::models::response::BResponse;
use crate::req_models::requests::AddWallet;
use crate::services::profile_service::{MiniProfile, ProfileWithFriends};
use crate::services::tcp::send_to_tcp_server;
use crate::services::user_service::UserService;
use crate::utils::formatter;
use crate::utils::vcrypto::{generate_compressed_pubkey, get_transaction_hash, sign_transaction};

#[get("/get_friends")]
pub async fn get_friends(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut resp_data = GenericResp::<ProfileWithFriends>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data:None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };

    // get by username
    let profile = match ProfileService::get_profile_with_friend(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Error getting user profile".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(profile);
    return HttpResponse::Ok().json(resp_data)
}


#[derive(Deserialize)]
struct GetUserProfilePath{
    pub username:String
}

#[get("/get/{username}")]
pub async fn get_user_profile(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>,
    path: web::Path<GetUserProfilePath>
)->HttpResponse{
    let mut resp_data = GenericResp::<ProfileWithFriends>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };

    // get by username
    let profile = match ProfileService::get_profile_with_friend(&pool, path.username.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Error getting user profile".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(profile);
    return HttpResponse::Ok().json(resp_data)
}

#[get("/get")]
pub async fn get_profile(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut resp_data = GenericResp::<ProfileWithFriends>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(ProfileWithFriends::default())
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    
    // get by username
    let profile = match ProfileService::get_profile_with_friend(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Error getting user profile".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(profile);
    return HttpResponse::Ok().json(resp_data)
}



#[derive(Deserialize)]
struct SearchPath{
    pub data:String
}

#[get("/search/{data}")]
pub async fn search(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>,
    path: web::Path<SearchPath>
)->HttpResponse{
    let mut resp_data = GenericResp::<Vec<MiniProfile>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };

    // get by username
    let profile = match ProfileService::search_users(&pool, path.data.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Error getting search result".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(profile);
    return HttpResponse::Ok().json(resp_data)
}


#[post("/update")]
pub async fn update_profile(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut resp_data = GenericResp::<Profile>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(Profile::default())
    };
    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data); 
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };

    // get profile 
    let ex_profile =match  ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error fetching profile data  {}", err.to_string());
            resp_data.message = "Error getting profile data".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;

            return HttpResponse::InternalServerError().json(resp_data)
        }
    };



    // create update profile 
    let mut profile = ex_profile.clone();
    if req.bio.is_some(){
        profile.bio = Some(req.bio.clone().unwrap());
    }
    if req.image.is_some(){
        profile.image = Some(req.image.clone().unwrap());
    }
   
    if req.name.is_some(){
        profile.name = Some(req.name.clone().unwrap());
    }
    if req.app_f_token.is_some(){
        profile.app_f_token = Some(req.app_f_token.clone().unwrap());
    }
    if req.earnings_wallet.is_some(){
        profile.earnings_wallet = Some(req.earnings_wallet.clone().unwrap());
    }
    
    // update 
    match ProfileService::update_profile(&pool, profile.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error updating profile data  {}", err.to_string());
            resp_data.message = "Error updating profile data".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;

            return HttpResponse::InternalServerError().json(resp_data)   
        }
    };
    resp_data.message = "Ok".to_string();
    resp_data.server_message =None;
    resp_data.data = Some(profile);
    return HttpResponse::Ok().json(resp_data);
}



// delete my account
// #[get("/delete")]
// pub async fn delete_profile(
//     pool:Data<DbPool>,
//     claim:Option<ReqData<Claims>>
// )->HttpResponse{
//     let mut resp_data = GenericResp::<Profile>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: Some(Profile::default())
//     };
// 
//     //claim
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             resp_data.message = "Unauthorized".to_string();
// 
//             return HttpResponse::Unauthorized()
//                 .json(
//                     resp_data
//                 )
//         }
//     };
//     
//     
// 
//     let ex_profile =match  UserService::get_by_username (&pool, claim.user_name.clone()).await{
//         Ok(data)=>{data},
//         Err(err)=>{
//             log::error!(" error fetching profile data  {}", err.to_string());
//             resp_data.message = "Error getting profile data".to_string();
//             resp_data.server_message = Some(err.to_string());
//             resp_data.data = None;
// 
//             return HttpResponse::InternalServerError().json(resp_data)
//         }
//     };
//     
//     let mut user = ex_profile.clone();
//     user.is_deleted = true;
//     
//     // delete account 
//     match ProfileService::(&database.db, claim.user_name.clone()).await{
//         Ok(_)=>{},
//         Err(err)=>{
//             log::error!(" error deleting profile data  {}", err.to_string());
//             resp_data.message = "Error deleting profile data".to_string();
//             resp_data.server_message = Some(err.to_string());
//             resp_data.data = None;
//             return HttpResponse::InternalServerError().json(resp_data) 
//         }
//     }
// 
//     resp_data.message = "Your profile has been deleted successfully".to_string();
//     resp_data.server_message = None;
//     resp_data.data = None;
//     return HttpResponse::Ok().json(resp_data)
// 
// }

#[get("/friend_suggestion")]
pub async fn get_friend_suggestion(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut resp_data = GenericResp::<Vec<MiniProfile>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    
    let suggestions = match ProfileService::friend_suggestion(&pool).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("Error getting friend suggestion... {}", err);
            resp_data.message = "Error getting friend suggestion".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data);
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(suggestions);
    return HttpResponse::Ok().json(resp_data);
}

#[get("/samp")]
pub async fn sample_controller(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    return HttpResponse::Ok().json({})
}

// api to post earnings unspent earnings 
#[post("/update_earnings")]
pub async fn post_earnings(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    
    // get profile 
    let profile =match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("Error getting profile {}", err.to_string());
            resp_data.message = "Error getting profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    // drop if user has not referred anyone
    if profile.referred_users.len() < 1{
        resp_data.message = "You have not referred anyone".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data)
    }
    if !profile.is_earnings_activated{
        resp_data.message = "You have not activated earnings".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data) 
    }
    
    // update earnings
    let earnings = match BigDecimal::from_str(req.new_earning.to_owned().unwrap_or_default().as_str()){
        Ok(earnings)=>{earnings},
        Err(err)=>{
            error!("{}", err);
            resp_data.message = "Error with data".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };
    let mut new_profile = profile.clone();
    new_profile.unclaimed_earnings = profile.unclaimed_earnings.clone() + earnings ;
    match ProfileService::update_profile(&pool, new_profile).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("{}", err);
            resp_data.message = "Error updating profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data) 
        }
    };
    
    resp_data.message = "OK".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}

// api to cash out earnings
#[get("/cashout_earnings")]
pub async fn cashout_earnings(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    // get profile
    let profile =match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("Error getting profile {}", err.to_string());
            resp_data.message = "Error getting profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };
    
    let earnings_wallet = match profile.earnings_wallet.to_owned(){
        Some(earnings)=>{earnings},
        None=>{
            resp_data.message = "No earnings wallet selected".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::BadRequest().json( resp_data);  
        }
    };

    // send coins on the blockchain
    let sender =  CONFIG.earnings_wallet.to_owned();
    let timestamp =  Local::now().naive_local().timestamp() as u64;
    let mut message = BRequest{
        action: "transfer".to_string(),
        data: BTransfer{
            sender:  sender.to_owned(),
            receiver: earnings_wallet.to_owned() ,
            amount: profile.unclaimed_earnings.to_owned(),
            timestamp: 0,
            id: "".to_string(),
            signature: "".to_string(),
        },
    };
    message.data.timestamp = timestamp;
    let tx_hash = get_transaction_hash(message.to_owned().data);
    let (priv_key, _) = generate_compressed_pubkey(CONFIG.earnings_wallet_password.to_owned().as_str());
    message.data.id = tx_hash.to_owned();
    let sig = match sign_transaction(
        sender.as_str(),
        message.data.receiver.as_str(),
        message.data.amount.normalized().to_string().as_str(),
        timestamp,
        message.data.id.as_str(),
        priv_key
    ){
        Ok(sig)=>{sig},
        Err(err)=>{
            error!("{}", err);
            resp_data.message = "Error with signing transaction".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };
    message.data.signature = sig;
    let message_string = serde_json::to_string(&message).unwrap_or_default();
    debug!("request to blockchain {}", message_string);
    let result = web::block(move || send_to_tcp_server(message_string,CONFIG.blockchain_ip.to_owned() )).await;
    let res_data = match result {
        Ok(data)=>{
            match data {
                Ok(data)=>{data},
                Err(err)=>{
                    error!("{}", err);
                    resp_data.message = "Error with request".to_string();
                    resp_data.server_message = None;
                    resp_data.data = None;
                    return HttpResponse::InternalServerError().json(resp_data)
                }
            }
        },
        Err(err)=>{
            error!("blockchian message error {}", err);
            resp_data.message = "Error with request".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };
    // perse result
    let res:BResponse<String> = serde_json::from_str(&res_data).unwrap_or_default();
    
    if res.status == 0 {
        error!("{}", res.message);
        resp_data.message = "transaction failed".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data)
    }
    let mut new_profile = profile.clone();
    new_profile.unclaimed_earnings = BigDecimal::zero();
    match ProfileService::update_profile(&pool, new_profile).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("error updating profile {}", err);
            resp_data.message = "Error updating profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}

// api to activate earnings 
#[get("/activate_earnings")]
pub async fn activate_earnings(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    // get profile
    let profile =match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("Error getting profile {}", err.to_string());
            resp_data.message = "Error getting profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };
    // drop if user has not referred anyone
    if profile.referred_users.len() < 1{
        resp_data.message = "You have not referred anyone".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data)  
    }

    let mut new_profile = profile.clone();
    if new_profile.is_earnings_activated{
        new_profile.is_earnings_activated = false
    }else{
        new_profile.is_earnings_activated = true
    }
    match ProfileService::update_profile(&pool, new_profile).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("error updating profile {}", err);
            resp_data.message = "Error updating profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}
// update referrals
#[post("/update_referrals")]
pub async fn update_referrals(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    // get profile
    let profile =match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("Error getting profile {}", err.to_string());
            resp_data.message = "Error getting profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let mut new_profile = profile.clone();
    let new_ref = match req.new_referrals.to_owned(){
        Some(new_ref)=>{new_ref},
        None=>{
            resp_data.message = "No referral sent".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::BadRequest().json(resp_data)
        }
    };
    new_profile.referred_users.extend(new_ref);
    match ProfileService::update_profile(&pool, new_profile).await{
        Ok(data)=>{data},
        Err(err)=>{
            error!("error updating profile {}", err);
            resp_data.message = "Error updating profile".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}




#[post("/add_wallet")]
pub async fn add_wallet(
    pool:Data<PgPool>,
    req: Result<web::Json<AddWallet>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    
    // get user profile 
    let profile = match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(profile)=>{profile},
        Err(err)=>{
            log::error!("errror getting user profile .. {}", err);
            resp_data.message = "Error getting user profile".to_string();
            resp_data.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            println!("{}", err.to_string());
            resp_data.message = "Error sending data to blockchain".to_string();
            resp_data.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };
    
    // verify from the blockchain 
    let ver_data = BVerifyWallet{
        address:req.address.to_owned(),
        message:req.message.to_owned(),
        signature:req.signature.to_owned(),
    };
    let message = BRequest::<BVerifyWallet>{
        action:"verify_wallet".to_string(),
        data:ver_data,
    };
    let message_data = match serde_json::to_string(&message){
        Ok(data)=>{data},
        Err(err)=>{
            println!("{}", err.to_string());
            resp_data.message = "Error sending data to blockchain".to_string();
            resp_data.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };
   

    let ip = kura_coin_server_ip.clone();

    let result = web::block(move || send_to_tcp_server(message_data.clone(),ip  )).await;
    let response_string =match result {
        Ok(data)=>{
            match data {
                Ok(data)=>{data},
                Err(err)=>{
                    println!("{}", err.to_string());
                    resp_data.message = "Error from blockchain".to_string();
                    resp_data.server_message = Some(err.to_string());
                    return HttpResponse::InternalServerError().json( resp_data);
                }
            }
        },
        Err(err)=>{
            println!("{}", err.to_string());
            resp_data.message = "Error from blockchain".to_string();
            resp_data.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let rdata= match from_str::<BResponse::<String>>(response_string.as_str()){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error decoding response {}", err);
            resp_data.message = "Data Error. Error validating wallet ".to_string();
            resp_data.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(resp_data);
        }
    };

    if rdata.status != 1{
        // blockchain request failed
        resp_data.message = resp_data.message.to_owned();
        resp_data.server_message =Some(rdata.message);
        return HttpResponse::BadRequest().json( resp_data);
    }
    
    
    // update wallet
    let mut new_profile = profile.clone();
    let wallets = profile.wallets.clone().unwrap_or_default();
   
    if profile.wallets.is_none(){
        new_profile.wallets  = Some(req.address.to_owned())
    }else{
        let mut cloned_profile = profile.clone();
        let wallets: Vec<&str> = match cloned_profile.wallets.as_deref() {
            Some(w) => w.split(',').collect(),
            None => Vec::new(),  // Or handle error differently
        };
        if wallets.contains(&&*req.address.to_owned()){
            resp_data.message = "Wallet already exists".to_string();
            resp_data.server_message = None;
            return HttpResponse::BadRequest().json(resp_data)  
        }
        new_profile.wallets = Some(format!("{},{}",profile.wallets.clone().unwrap_or_default(), req.address.to_owned()));
    }
    
    match ProfileService::update_profile(&pool, new_profile).await{
        Ok(_)=>{},
        Err(err)=>{
            println!("{}", err.to_string());
            resp_data.message = "Error updating wallets".to_string();
            resp_data.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };
    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    return HttpResponse::Ok().json(resp_data)
}

