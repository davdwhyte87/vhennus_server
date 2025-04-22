use std::env;
use std::io::BufRead;
use actix_web::{get, post, web::{self, Data, ReqData}, HttpResponse, ResponseError};
use serde::Deserialize;
use serde_json::from_str;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{models::{profile::Profile, response::GenericResp}, req_models::{create_sell_order_req::CreatePostReq, requests::UpdateProfileReq}, services::{mongo_service::MongoService, profile_service::ProfileService}, utils::auth::Claims};
use crate::controllers::user_controller::resend_code;
use crate::models::request_models::{BRequest, BVerifyWallet, TransferReq};
use crate::models::response::BResponse;
use crate::req_models::requests::AddWallet;
use crate::services::profile_service::{MiniProfile, ProfileWithFriends};
use crate::services::tcp::send_to_tcp_server;
use crate::services::user_service::UserService;
use crate::utils::formatter;

#[get("/get_friends")]
pub async fn get_friends(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<ProfileWithFriends>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data:None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // get by username
    let profile = match ProfileService::get_profile_with_friend(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting user profile".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(profile);
    return HttpResponse::Ok().json(respData)
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
    let mut respData = GenericResp::<ProfileWithFriends>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // get by username
    let profile = match ProfileService::get_profile_with_friend(&pool, path.username.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting user profile".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(profile);
    return HttpResponse::Ok().json(respData)
}

#[get("/get")]
pub async fn get_profile(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<ProfileWithFriends>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(ProfileWithFriends::default())
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };
    
    // get by username
    let profile = match ProfileService::get_profile_with_friend(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting user profile".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(profile);
    return HttpResponse::Ok().json(respData)
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
    let mut respData = GenericResp::<Vec<MiniProfile>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // get by username
    let profile = match ProfileService::search_users(&pool, path.data.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting search result".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(profile);
    return HttpResponse::Ok().json(respData)
}


#[post("/update")]
pub async fn update_profile(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Profile>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(Profile::default())
    };
    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData); 
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // get profile 
    let ex_profile =match  ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error fetching profile data  {}", err.to_string());
            respData.message = "Error getting profile data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData)
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
    
    // update 
    match ProfileService::update_profile(&pool, profile.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error updating profile data  {}", err.to_string());
            respData.message = "Error updating profile data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData)   
        }
    };
    respData.message = "Ok".to_string();
    respData.server_message =None;
    respData.data = Some(profile);
    return HttpResponse::Ok().json(respData);
}



// delete my account
// #[get("/delete")]
// pub async fn delete_profile(
//     pool:Data<DbPool>,
//     claim:Option<ReqData<Claims>>
// )->HttpResponse{
//     let mut respData = GenericResp::<Profile>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: Some(Profile::default())
//     };
// 
//     //claim
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             respData.message = "Unauthorized".to_string();
// 
//             return HttpResponse::Unauthorized()
//                 .json(
//                     respData
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
//             respData.message = "Error getting profile data".to_string();
//             respData.server_message = Some(err.to_string());
//             respData.data = None;
// 
//             return HttpResponse::InternalServerError().json(respData)
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
//             respData.message = "Error deleting profile data".to_string();
//             respData.server_message = Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::InternalServerError().json(respData) 
//         }
//     }
// 
//     respData.message = "Your profile has been deleted successfully".to_string();
//     respData.server_message = None;
//     respData.data = None;
//     return HttpResponse::Ok().json(respData)
// 
// }

#[get("/friend_suggestion")]
pub async fn get_friend_suggestion(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Vec<MiniProfile>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    
    let suggestions = match ProfileService::friend_suggestion(&pool).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("Error getting friend suggestion... {}", err);
            respData.message = "Error getting friend suggestion".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(suggestions);
    return HttpResponse::Ok().json(respData);
}

#[get("/samp")]
pub async fn sample_controller(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };
    return HttpResponse::Ok().json({})
}

#[post("/add_wallet")]
pub async fn add_wallet(
    pool:Data<PgPool>,
    req: Result<web::Json<AddWallet>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };
    
    // get user profile 
    let profile = match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(profile)=>{profile},
        Err(err)=>{
            log::error!("errror getting user profile .. {}", err);
            respData.message = "Error getting user profile".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( respData);
        }
    };

    let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            println!("{}", err.to_string());
            respData.message = "Error sending data to blockchain".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( respData);
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
            respData.message = "Error sending data to blockchain".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( respData);
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
                    respData.message = "Error from blockchain".to_string();
                    respData.server_message = Some(err.to_string());
                    return HttpResponse::InternalServerError().json( respData);
                }
            }
        },
        Err(err)=>{
            println!("{}", err.to_string());
            respData.message = "Error from blockchain".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json( respData);
        }
    };

    let resp_data= match from_str::<BResponse::<String>>(response_string.as_str()){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error decoding response {}", err);
            respData.message = "Data Error. Error validating wallet ".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(respData);
        }
    };

    if resp_data.status != 1{
        // blockchain request failed
        respData.message = "Authentication error".to_string();
        respData.server_message =None;
        return HttpResponse::BadRequest().json( respData);
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
            respData.message = "Wallet already exists".to_string();
            respData.server_message = None;
            return HttpResponse::BadRequest().json(respData)  
        }
        new_profile.wallets = Some(format!("{},{}",profile.wallets.clone().unwrap_or_default(), req.address.to_owned()));
    }
    
    match ProfileService::update_profile(&pool, new_profile).await{
        Ok(_)=>{},
        Err(err)=>{
            println!("{}", err.to_string());
            respData.message = "Error updating wallets".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    respData.message = "Ok".to_string();
    respData.server_message = None;
    return HttpResponse::Ok().json(respData)
}

