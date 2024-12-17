use actix_web::{cookie::time::error, dev::Path, get, post, web::{self, Data, ReqData}, HttpResponse};
use mongodb::bson::doc;
use serde::Deserialize;

use crate::{models::{chat::Chat, chat_pair::ChatPair, circle::Circle, request_models::{CreateChatReq, CreateGroupChatReq}, response::GenericResp}, services::{chat_pair_service::ChatPairService, chat_service::ChatService, circle_service::CircleService, mongo_service::MongoService, user_service::UserService}, utils::{auth::Claims, general::get_current_time_stamp}};


#[post("/create")]
pub async fn create_chat(
    database:Data<MongoService>,
    req: Result<web::Json<CreateChatReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Chat>{
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

    let mut chat = Chat{
        id: uuid::Uuid::new_v4().to_string(),
        pair_id: req.pair_id.clone(),
        sender: claim.user_name.clone(),
        receiver: req.receiver.clone(),
        message: "".to_string(),
        image: "".to_string(),
        created_at: get_current_time_stamp(),
        updated_at:get_current_time_stamp()
    };
    if req.message.is_some(){
        chat.message = req.message.clone().unwrap_or_default()
    }
    if req.image.is_some(){
        chat.image = req.image.clone().unwrap_or_default()
    }

    
    match ChatService::create_chat(&database.db, &chat).await{
        Ok(_)=>{},
        Err(err)=>{
            log::error!("{}", err)
        }
    };

    return HttpResponse::Ok().json({})
}


#[derive(Deserialize)]
struct ChatPath {
    id: String,
}

#[derive(Deserialize)]
struct ChatPathName {
    name: String,
}


#[get("/get_pair/{id}")]
pub async fn get_by_pair(
    database:Data<MongoService>,
    path: web::Path<ChatPath>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<Chat>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    
    let chats = match ChatService::get_chats_by_pair_id(&database.db, path.id.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error getting chats {}", err);
            respData.message = "error getting chats".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData); 
        }
    };

    respData.message = "ok".to_string();
    respData.server_message = None;
    respData.data = Some(chats);
    return HttpResponse::Ok().json(respData)
}

#[get("/get_all_chats")]
pub async fn get_all_chats(
    claim:Option<ReqData<Claims>>,
    database:Data<MongoService>,
)->HttpResponse{
    let mut respData = GenericResp::<Vec<Chat>>{
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

    let chats = match ChatService::get_user_chats(&database.db, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error getting chats {}", err.to_string());
            respData.message = "error getting chats".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData); 
        }
    };
    respData.message = "ok".to_string();
    respData.server_message = None;
    respData.data = Some(chats);
    return HttpResponse::Ok().json(respData)
}


#[derive(Deserialize)]
pub struct UserNamePath{
    pub user_name:String
}
#[get("/create_chat_pair/{user_name}")]
pub async fn create_chat_pair(
    database:Data<MongoService>,
    path: web::Path<UserNamePath>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<ChatPair>{
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

    //check if the username is correct 
    match UserService::get_by_(&database.db, doc! {"user_name":path.user_name.clone()}).await{
        Ok(data)=>{
            match data{
                Some(_)=>{},
                None=>{
                    respData.message = "User is not found".to_string();
                    respData.server_message = None;
                    respData.data = None;
                    return HttpResponse::BadRequest().json( respData);    
                }
            }
        },
        Err(err)=>{
            log::error!("error getting suer   {}", err.to_string());
            respData.message = "Error getting user".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);     
        }
    }

    let chat_pair = ChatPair{
        id : uuid::Uuid::new_v4().to_string(),
        user_name: claim.user_name.clone(),
        users_ids: vec![claim.user_name.clone(), path.user_name.clone()],
        users: None,
        last_message : "".to_string(),
        all_read: true,
        created_at: get_current_time_stamp(),
        updated_at: get_current_time_stamp()
    };

    let res = match ChatPairService::create_chat_pair(&database.db, &chat_pair).await {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error creating chat  {}", err.to_string());
            respData.message = "Error creating chat pair".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);    
        }
    };

    respData.message = "ok".to_string();
    respData.server_message = None;
    respData.data = Some(res);
    return HttpResponse::Ok().json(respData)
}


#[get("/get_my_chat_pairs")]
pub async fn get_my_chat_pairs(
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Vec<ChatPair>>{
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



    let res = match ChatPairService::get_all_my_chat_pairs(&database.db, claim.user_name.clone()).await {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error getting chats  {}", err.to_string());
            respData.message = "Error getting chat pairs".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);    
        }
    };

    respData.message = "ok".to_string();
    respData.server_message = None;
    respData.data = Some(res);
    return HttpResponse::Ok().json(respData)
}

#[post("/create")]
pub async fn create_group_chat(
    database:Data<MongoService>,
    req: Result<web::Json<CreateGroupChatReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{

    let mut respData = GenericResp::<Chat>{
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

    // check if members are empty 
    if req.members.is_empty() {
        respData.message = "Validation error, Circle must have a member".to_string();
        respData.server_message = Some("Circle must have a member".to_string());
        respData.data = None;
        return HttpResponse::BadRequest().json(respData)
    }

    
    let mut circle = Circle{
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        display_name: req.display_name.clone(),
        owner: claim.user_name.clone(),
        members: req.members.clone(),
        is_private: false,
        image: req.image.clone(),
        created_at: get_current_time_stamp(),
        updated_at:get_current_time_stamp()
    };
 

    
    match CircleService::create_circle(&database.db, &circle).await{
        Ok(_)=>{},
        Err(err)=>{
            log::error!("{}", err)
        }
    };

    return HttpResponse::Ok().json({})
}


//get group chats
#[get("/get_chats/{name}")]
pub async fn get_group_chats(
    database:Data<MongoService>,
    path: web::Path<ChatPathName>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
   
    let mut respData = GenericResp::<Vec<Chat>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let chats = match ChatService::get_chats_by_pair_id(&database.db, path.name.clone()).await{
        Ok(data)=>{
            data
        },
        Err(err)=>{
            log::error!("{}", err);
            respData.message = "Error getting chats data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);   
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(chats);
    return HttpResponse::Ok().json(respData)

}

//get circle
#[get("/get_circle/{name}")]
pub async fn get_circle(
    database:Data<MongoService>,
    path: web::Path<ChatPathName>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Circle>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let circle = match CircleService::get_circle(&database.db, path.name.clone()).await{
        Ok(data)=>{
            match data{
                Some(data)=>{data},
                None=>{
                    respData.message = "No circle found".to_string();
                    respData.server_message = None;
                    respData.data = None;
                    return HttpResponse::BadRequest().json(respData);  
                }
            }
        },
        Err(err)=>{
            log::error!("{}", err);
            respData.message = "Error getting circle data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(circle);
    return HttpResponse::Ok().json(respData)

}

// create group chat


