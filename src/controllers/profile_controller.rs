
use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse, ResponseError};

use crate::{models::{profile::Profile, response::GenericResp}, req_models::{create_sell_order_req::CreatePostReq, requests::UpdateProfileReq}, services::{mongo_service::MongoService, profile_service::ProfileService}, utils::auth::Claims};


#[get("/get_friends")]
pub async fn get_friends(
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
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
    let profile = match ProfileService::get_friends(&database.db, claim.user_name.clone()).await{
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
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
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
    let profile = match ProfileService::get_profile(&database.db, claim.user_name.clone()).await{
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

#[post("/update")]
pub async fn update_profile(
    database:Data<MongoService>,
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
    let ex_profile =match  ProfileService::get_profile(&database.db, claim.user_name.clone()).await{
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
        profile.bio = req.bio.clone().unwrap();
    }
    if req.image.is_some(){
        profile.image = req.image.clone().unwrap();
    }
    if req.occupation.is_some(){
        profile.occupation = req.occupation.clone().unwrap();
    }
    if req.work.is_some(){
        profile.work = req.work.clone().unwrap();
    }

    profile.user_name = claim.user_name.clone();
    profile.id = uuid::Uuid::new_v4().to_string();
    // update 
    match ProfileService::update_profile(&database.db, &profile).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error updating profile data  {}", err.to_string());
            respData.message = "Error updating profile data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData)   
        }
    };
    return HttpResponse::Ok().json({});
}




