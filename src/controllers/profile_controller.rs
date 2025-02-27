
use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::{models::{profile::Profile, response::GenericResp}, req_models::{create_sell_order_req::CreatePostReq, requests::UpdateProfileReq}, services::{mongo_service::MongoService, profile_service::ProfileService}, utils::auth::Claims, DbPool};
use crate::services::profile_service::{MiniProfile, ProfileWithFriends};
use crate::services::user_service::UserService;

#[get("/get_friends")]
pub async fn get_friends(
    pool:Data<DbPool>,
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
    pool:Data<DbPool>,
    claim:Option<ReqData<Claims>>,
    path: web::Path<GetUserProfilePath>
)->HttpResponse{
    let mut respData = GenericResp::<Profile>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(Profile::default())
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
    let profile = match ProfileService::get_profile(&pool, path.username.clone()).await{
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
    pool:Data<DbPool>,
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
    pool:Data<DbPool>,
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
    pool:Data<DbPool>,
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