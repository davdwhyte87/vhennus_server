use std::{env, fmt, str::FromStr};

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse, ResponseError};
use actix_web_validator::Json;
use bigdecimal::{num_bigint::BigInt, BigDecimal};
use gcp_auth::provider;
use log::error;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::{controllers::buy_order_controller::escrow_to_user, models::{comment::Comment, payment_method::PaymentMethod, post::Post, request_models::TransferReq, response::{GenericResp, Response}, sell_order::{self, Currency, SellOrder}}, req_models::create_sell_order_req::{CreateCommentReq, CreatePostReq, CreateSellOrderReq, UpdateSellOrderReq}, services::{mongo_service::MongoService, post_service::PostService, sell_order_service::SellOrderService, tcp::send_to_tcp_server}, utils::{auth::Claims, formatter}};
use crate::services::post_service::{PostFeed, PostWithComments};
use crate::services::profile_service::ProfileService;
use crate::utils::general::get_time_naive;

#[post("/create")]
pub async fn create_post(
    pool:Data<PgPool>,
     req: Result<web::Json<CreatePostReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Post>{
        message:"".to_string(),
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

    // post model
    let mut new_post = Post{
        id: Uuid::new_v4().to_string(),
        text: req.text.to_owned(),
        image: None,
        created_at: get_time_naive(),
        updated_at: get_time_naive(),
        user_name:claim.user_name.to_owned(),
    };
    
    if req.image.is_some(){
        new_post.image = Some(req.image.to_owned().unwrap())
    }


    match PostService::create_post(&pool, new_post).await {
        Ok(_)=>{},
        Err(err)=>{
            log::error!(" error creating post {}", err.to_string());
            respData.message = "Error creating post".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);
             
        }
    };
    
    // silently try to pay the user 
    let profile =match ProfileService::get_profile(&pool, claim.user_name.clone()).await{
        Ok(profile)=>{Some(profile)},
        Err(err)=>{
            error!("error getting profile {}", err);
            None
        }
    };
    if profile.is_some(){
        let mut new_profile = profile.clone().unwrap();
        if new_profile.is_earnings_activated{
            let post_amount = BigDecimal::from_str("62").unwrap_or_default();
            new_profile.unclaimed_earnings = new_profile.unclaimed_earnings+post_amount; 
            ProfileService::update_profile(&pool, new_profile).await;
        }
    }
    
    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = None;
    return HttpResponse::Ok().json( respData);
}



#[get("/all")]
pub async fn get_all_posts(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<PostFeed>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    println!("new req");

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

    let posts = match PostService::get_all_post(&pool).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting posts {}", err.to_string());
            respData.message = "Error getting posts".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json( respData);
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json( respData);
    
}


#[get("/allmy")]
pub async fn get_my_posts(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<PostFeed>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    println!("new req");

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

    let posts = match PostService::get_all_my_posts(&pool, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting posts {}", err.to_string());
            respData.message = "Error getting posts".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json( respData);
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json( respData);
    
}


#[derive(Deserialize, Serialize, Debug, Clone)]
struct GetUsersPosts{
    user_name:String,
}
#[get("/all/{user_name}")]
pub async fn get_users_posts(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>,
    path: web::Path<GetUsersPosts>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<PostFeed>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    println!("new req");

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

    let posts = match PostService::get_user_posts(&pool, path.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting posts {}", err.to_string());
            respData.message = "Error getting posts".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json( respData);
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json( respData);

}


#[derive(Deserialize)]
struct GetSinglePostPath {
    id: String,
}

#[post("/{id}/comment/create")]
pub async fn create_comment(
    pool:Data<PgPool>,
    req:Json<CreateCommentReq>,
    claim:Option<ReqData<Claims>>,
    path: web::Path<GetSinglePostPath>
)->HttpResponse{

    let mut respData = GenericResp::<Comment>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(Comment::default())
    };
    println!("new req");

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

    // post model
    let new_comment = Comment{
        id: Uuid::new_v4().to_string(),
        text: req.text.to_owned(),
        created_at: get_time_naive(),
        user_name:claim.user_name.to_owned(),
        post_id: path.id.to_owned()
    };

    match PostService::create_comment(&pool, new_comment.clone()).await {
        Ok(_)=>{},
        Err(err)=>{
            log::error!(" error creating comment {}", err.to_string());
            respData.message = "Error creating comment".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);
             
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(new_comment);
    return HttpResponse::Ok().json( respData);
}


#[get("/single/{id}")]
pub async fn get_single_posts(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>,
    path:web::Path<GetSinglePostPath>
)->HttpResponse{

    let mut respData = GenericResp::<PostWithComments>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    println!("new req");

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

    let posts = match PostService::get_single_post(&pool, path.id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting post {}", err.to_string());
            respData.message = "Error getting post".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json( respData);
        }
    };
    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json( respData);
}


#[get("/like/{id}")]
pub async fn like_post(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>,
    path:web::Path<GetSinglePostPath>
)->HttpResponse{
    let mut respData = GenericResp::<Post>{
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


    

    match PostService::toggle_like(&pool, path.id.clone(),claim.user_name.clone()).await{
        Ok(_)=>{},
        Err(err)=>{
            log::error!("error updating post {}", err.to_string());
            respData.message = "Error saving data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::NotFound().json( respData); 
        }
    };
    respData.message = "OK".to_string();
    respData.server_message = None;
    respData.data = None;
    return HttpResponse::Ok().json( respData);
}




#[derive(Debug, Serialize)]
struct ApiError {
    error:String,
    message: String,
   
}

// impl fmt::Display for ApiError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.message)
//     }
// }

// impl ResponseError for ApiError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::build(self.status_code()).json(self)
//     }
//
//     fn status_code(&self) -> actix_web::http::StatusCode {
//         actix_web::http::StatusCode::BAD_REQUEST
//     }
// }