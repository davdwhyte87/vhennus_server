use std::{env, fmt, str::FromStr};

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse, ResponseError};
use actix_web_validator::Json;
use bigdecimal::{num_bigint::BigInt, BigDecimal};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::{controllers::buy_order_controller::escrow_to_user, models::{comment::Comment, payment_method::PaymentMethod, post::Post, request_models::TransferReq, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}}, req_models::create_sell_order_req::{ CreateCommentReq, CreatePostReq, CreateSellOrderReq, UpdateSellOrderReq}, services::{mongo_service::MongoService, post_service::PostService, sell_order_service::SellOrderService, tcp::send_to_tcp_server}, utils::{auth::Claims, formatter}};




#[post("/create")]
pub async fn create_post(
    database:Data<MongoService>,
     req: Result<web::Json<CreatePostReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Post>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(Post::default())
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
    // match req.validate(){
    //     Ok(_)=>{},
    //     Err(err)=>{
           
    //         return HttpResponse::BadRequest().json( err); 
    //     }

   

    println!("create post request");

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
    let new_post = Post{
        id: Uuid::new_v4().to_string(),
        text: req.text.to_owned(),
        image: "".to_string(),
        created_at: chrono::offset::Utc::now().to_string(),
        user_name:claim.user_name.to_owned(),
        likes : vec![],
        comments_ids: vec![],
        comments: None,
        number_of_views: 100

    };


    match PostService::create_post(&database.db, &new_post).await {
        Ok(_)=>{},
        Err(err)=>{
            log::error!(" error creating post {}", err.to_string());
            respData.message = "Error creating post".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);
             
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(new_post);
    return HttpResponse::Ok().json( respData);
}



#[get("/all")]
pub async fn get_all_posts(
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<Post>>{
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

    let posts = match PostService::get_all_post(&database.db).await{
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
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<Post>>{
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

    let posts = match PostService::get_all_my_posts(&database.db, claim.user_name.clone()).await{
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
    database:Data<MongoService>,
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
        created_at: chrono::offset::Utc::now().to_string(),
        user_name:claim.user_name.to_owned(),
        post_id: path.id.to_owned()
    };

    match PostService::create_comment(&database.db, &new_comment).await {
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
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    path:web::Path<GetSinglePostPath>
)->HttpResponse{

    let mut respData = GenericResp::<Post>{
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

    let posts = match PostService::get_single_post(&database.db, path.id.to_owned()).await{
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
    database:Data<MongoService>,
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

    // get post 
    let mut post = match PostService::get_single_post(&database.db, path.id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error getting post {}", err.to_string());
            respData.message = "Post not found".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::NotFound().json( respData); 
        }
    };

    if !post.likes.contains(&claim.user_name){
        post.likes.push(claim.user_name.to_owned());
    }else{
        post.likes.retain(|x| x!=&claim.user_name)
    }
    

    match PostService::update_post(&database.db, post).await{
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

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }
}