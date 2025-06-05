use std::collections::HashMap;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse};
use actix_web::web::{Data, Json, ReqData};
use actix_ws::handle;
use log::{debug, error};
use sqlx::PgPool;
use uuid::Uuid;
use crate::groups::models::{Group, MyGroupsView, Room, RoomMembers, RoomWithMembersView, UserRoomSessions};
use crate::groups::service::GroupService;
use crate::models::app_error::AppError;
use crate::models::response::GenericResp;
use crate::req_models::requests::{CreateGroupReq, CreateRoomReq, UpdateGroupReq, UpdateRoomReq};
use crate::services::chat_session_service::{chat_ws_service, UserConnections};
use crate::services::profile_service::MiniProfile;
use crate::utils::auth::Claims;
use crate::utils::general::get_time_naive;

#[post("/create_group")]
pub async fn create_group(
    pool:Data<PgPool>,
    body: web::Json<CreateGroupReq>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req: CreateGroupReq = body.into_inner();
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::create_group(&req,&pool, &claims).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:&str ="Error creating group";

            match err {
                AppError::AlreadyExistsError=>{
                    message = "Group already exists";
                }
                other =>{
                    message = "Error creating group";
                }
            }
            resp_data.message = message.to_owned();
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

#[post("/create_room")]
pub async fn create_room(
    pool:Data<PgPool>,
    body: Json<CreateRoomReq>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req: CreateRoomReq = body.into_inner();
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::create_room(&pool,&req, &claims).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:&str ="";
            match err {
                AppError::UnauthorizedError=>{
                    message = "You are not authorized to create this this room";
                },
                AppError::AlreadyExistsError=>{
                    message = "Room already exists";
                }
                AppError::NotFoundError(String, data)=>{
                    message = "Group does not exist";
                },
                other=>{
                    error!("{}", other.to_string());
                    message = "Error creating room";
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Room created successfully".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}

#[post("/update_group")]
pub async fn update_group(
    pool:Data<PgPool>,
    body: Json<UpdateGroupReq>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let req: UpdateGroupReq = body.into_inner();
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::update_group(&pool, &claims,req).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:&str ="";
            match err {
                AppError::UnauthorizedError=>{
                    message = "You are not authorized to create this this room";
                },
                AppError::NotFoundError(String, data)=>{
                    message = "Group does not exist";
                },
                other=>{
                    error!("{}", other.to_string());
                    message = "Error updating group";
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Group updated successfully".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}

#[get("/join_room")]
pub async fn join_room(
    pool:Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let rm = "xxx".to_string();
    let room_id = query.get("room_id").unwrap_or(&rm);
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::join_room(&pool,&claims, room_id).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:&str ="";
            match err {
                AppError::UnauthorizedError=>{
                    message = "You are not authorized to join room";
                }
                AppError::NotFoundError(String, data)=>{
                    message = "room does not exist";
                },
                AppError::AlreadyExistsError=>{
                    message = "User already exists";
                }
                other=>{
                    message = "Error joining room";
                }
            }
            resp_data.message = message.to_owned();
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

#[get("/join_room_with_code")]
pub async fn join_room_with_code(
    pool:Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let rm = "xxx".to_string();
    let code = query.get("code").unwrap_or(&rm);
    let mut room_code:Option<String> = None;
    if code.as_str() != rm.as_str(){
        room_code = Some(code.to_owned());
    }
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::join_room_with_code(&pool,&claims, room_code).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:String="".to_string();
            match err {
                AppError::UnauthorizedError=>{
                    message = "You are not authorized to join room".to_string();
                }
                AppError::NotFoundError(String, data)=>{
                    message = "invalid code".to_string();
                },

                AppError::AlreadyExistsError=>{
                    message = "user already exists".to_string();
                },
                AppError::BadRequestError(data)=>{
                    message =format!("Bad request: {}", data);
                }
                other=>{
                    message = "Error joining room".to_string();
                }
            }
            resp_data.message = message.to_owned();
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

#[get("/generate_room_code")]
pub async fn generate_room_code(
    pool:Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<String> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let rm = "xxx".to_string();
    let room_id = query.get("room_id").unwrap_or(&rm);
    let claims: Claims = claim.to_owned().into_inner();

    let code =match GroupService::generate_code(&pool, room_id, &claims).await{
        Ok(data)=>{data},
        Err(err)=>{
            let mut message:String="".to_string();
            match err {
                AppError::UnauthorizedError=>{
                    message = "You are not authorized".to_string();
                }
                AppError::NotFoundError(String, data)=>{
                    message = "room does not exist".to_string();
                },
                other=>{
                    message = "Error creating room code".to_string();
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::BadRequest().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(code);
    return HttpResponse::Ok().json(resp_data)
}

#[post("/update_room")]
pub async fn update_room(
    pool:Data<PgPool>,
    body: Json<UpdateRoomReq>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let req: UpdateRoomReq = body.into_inner();
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::update_room(&pool, &claims, req).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:&str ="";
            match err {
                AppError::UnauthorizedError=>{
                    message = "You are not authorized to update this room";
                },
                AppError::NotFoundError(_, _)=>{
                    message = "Room does not exist";
                },
                other=>{
                    error!("{}", other.to_string());
                    message = "Error updating room";
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Room updated successfully".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}

#[get("/leave_room")]
pub async fn leave_room(
    pool:Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim:ReqData<Claims>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let rm = "xxx".to_string();
    let room_id = query.get("room_id").unwrap_or(&rm);
    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::leave_room(&pool, &claims, room_id).await{
        Ok(_)=>{},
        Err(err)=>{
            let mut message:&str ="";
            match err {
                AppError::NotFoundError(_, _)=>{
                    message = "Room does not exist";
                },
                other=>{
                    message = "Error leaving room";
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Successfully left room".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    return HttpResponse::Ok().json(resp_data)
}

#[get("/get_my_groups")]
pub async fn get_my_groups(
    pool: Data<PgPool>,
    claim: ReqData<Claims>
) -> HttpResponse {
    let mut resp_data = GenericResp::<Vec<MyGroupsView>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claims: Claims = claim.to_owned().into_inner();

    match GroupService::get_my_groups(&pool, &claims).await {
        Ok(groups) => {
            resp_data.message = "Ok".to_string();
            resp_data.server_message = None;
            resp_data.data = Some(groups);
            HttpResponse::Ok().json(resp_data)
        },
        Err(err) => {
            let message = "Error fetching groups";
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            HttpResponse::InternalServerError().json(resp_data)
        }
    }
}

#[get("/get_group")]
pub async fn get_group(
    pool: Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim: ReqData<Claims>
) -> HttpResponse {
    let mut resp_data = GenericResp::<MyGroupsView> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let default_id = "".to_string();
    let group_id = query.get("group_id").unwrap_or(&default_id);

    if group_id.is_empty() {
        resp_data.message = "Group ID is required".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data);
    }

    match GroupService::get_group_by_id(&pool, group_id.to_string()).await {
        Ok(group) => {
            resp_data.message = "Ok".to_string();
            resp_data.server_message = None;
            resp_data.data = Some(group);
            HttpResponse::Ok().json(resp_data)
        },
        Err(err) => {
            let mut message = "Error fetching group";
            match err {
                AppError::NotFoundError(_, _) => {
                    message = "Group not found";
                    return HttpResponse::NotFound().json(GenericResp::<MyGroupsView> {
                        message: message.to_string(),
                        server_message: None,
                        data: None
                    });
                },
                _ => {
                    error!("Error fetching group: {}", err);
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            HttpResponse::InternalServerError().json(resp_data)
        }
    }
}

#[get("/get_room")]
pub async fn get_roomb(
    pool: Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim: ReqData<Claims>
) -> HttpResponse {
    let mut resp_data = GenericResp::<RoomWithMembersView> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let default_id = "".to_string();
    let room_id = query.get("room_id").unwrap_or(&default_id);

    if room_id.is_empty() {
        resp_data.message = "Room ID is required".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data);
    }

    match GroupService::get_room_with_members(&pool, room_id.to_string()).await {
        Ok(room) => {
            resp_data.message = "Ok".to_string();
            resp_data.server_message = None;
            resp_data.data = Some(room);
            HttpResponse::Ok().json(resp_data)
        },
        Err(err) => {
            let mut message = "Error fetching room";
            match err {
                AppError::NotFoundError(_, _) => {
                    message = "Room not found";
                    return HttpResponse::NotFound().json(GenericResp::<RoomWithMembersView> {
                        message: message.to_string(),
                        server_message: None,
                        data: None
                    });
                },
                _ => {
                    error!("Error fetching room: {}", err);
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            HttpResponse::InternalServerError().json(resp_data)
        }
    }
}

#[get("/get_room")]
pub async fn get_room(
    pool: Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim: ReqData<Claims>
) -> HttpResponse {
    let mut resp_data = GenericResp::<RoomWithMembersView> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let default_id = "".to_string();
    let room_id = query.get("room_id").unwrap_or(&default_id);

    if room_id.is_empty() {
        resp_data.message = "Room ID is required".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data);
    }

    match GroupService::get_room_with_members(&pool, room_id.to_string()).await {
        Ok(room) => {
            resp_data.message = "Ok".to_string();
            resp_data.server_message = None;
            resp_data.data = Some(room);
            HttpResponse::Ok().json(resp_data)
        },
        Err(err) => {
            let mut message = "Error fetching room";
            match err {
                AppError::NotFoundError(_, _) => {
                    message = "Room not found";
                    return HttpResponse::NotFound().json(GenericResp::<RoomWithMembersView> {
                        message: message.to_string(),
                        server_message: None,
                        data: None
                    });
                },
                _ => {
                    error!("Error fetching room: {}", err);
                }
            }
            resp_data.message = message.to_owned();
            resp_data.server_message = None;
            resp_data.data = None;
            HttpResponse::InternalServerError().json(resp_data)
        }
    }
}


pub async fn connect_to_rooms(
    pool: Data<PgPool>,
    claim: ReqData<Claims>,
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<UserRoomSessions>,
    room_members: web::Data<RoomMembers>,
    query: web::Query<HashMap<String, String>>,
) ->  Result<HttpResponse, Error> {
    let mut resp_data = GenericResp::<RoomWithMembersView> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let mut room_id: String = query
        .get("room_id")
        .cloned()
        .unwrap_or_default();

    if room_id.is_empty() {
        resp_data.message = "Room ID is required".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return Ok(HttpResponse::BadRequest().json(resp_data));
    }


    let (response,session,
        mut msg_stream) =match  handle(&req, stream){
        Ok((response, session, msg_stream))
        => {(response, session, msg_stream)},
        Err(err)=>{
            error!("error {}", err);
            resp_data.message = "Error connecting".to_string();
            resp_data.server_message = None;
            resp_data.data = None;
            return Ok(HttpResponse::InternalServerError().json(resp_data));
        }

    };

    debug!("attempting to connect {} to room {}", claim.user_name.to_owned(), room_id.to_owned());
    actix_web::rt::spawn(async move {
        GroupService::group_chat_ws(
            session,
            msg_stream,
            claim.user_name.to_owned(),
            room_id.to_owned(),
            &claim,
            data,
            room_members,
            &pool).await;
    });

    Ok(response)
}
