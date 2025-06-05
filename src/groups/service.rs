use std::collections::HashMap;
use actix_web::{web, App, Error, HttpResponse};
use actix_web::web::ReqData;
use actix_ws::{Message, Session};
use bigdecimal::num_traits::clamp_min;
use futures_util::future::err;
use futures_util::StreamExt;
use log::{debug, error};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use sqlx::PgPool;
use uuid::Uuid;
use crate::groups::models::{Group, MessageType, MyGroupsView, Room, RoomChatReq, RoomMembers, RoomMessage, RoomWithMembersView, UserRoom, UserRoomSessions};
use crate::groups::repository::GroupRepo;
use crate::models::app_error::AppError;
use crate::models::chat::Chat;
use crate::models::request_models::CreateChatReq;
use crate::req_models::requests::{CreateGroupReq, CreateRoomReq, UpdateGroupReq, UpdateProfileReq, UpdateRoomReq};
use crate::services::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};
use crate::services::chat_service::ChatService;
use crate::services::chat_session_service::UserConnections;
use crate::services::profile_service::ProfileService;
use crate::utils::auth::Claims;
use crate::utils::general::get_time_naive;
use crate::utils::strings_stuff::truncate_string;

pub struct GroupService{

}
impl GroupService{
    pub async fn create_group(
        req: &CreateGroupReq,
        pool:&PgPool,
        claim:&Claims
    )->Result<(), AppError>{

        let group = Group{
            id: Uuid::new_v4().to_string(),
            user_name: claim.user_name.clone(),
            name: req.name.to_owned(),
            description: req.description.to_owned(),
            is_private: req.is_private,
            image: req.image.to_owned(),
            category: req.category.to_owned(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        match GroupRepo::create_group(pool, &group).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error creating group {}{}", group.name, err );
                return Err(err.into());
            }
        };
        return Ok(())
    }

    pub async fn create_room(
        pool:&PgPool,
        req:&CreateRoomReq,
        claim: &Claims
    )->Result<(), AppError>{
        // make sure the req user is the owner of the group
        // get the group 
        let group =  GroupRepo::get_group_by_id(pool, req.group_id.clone()).await
            .map_err(|err| err)?;

        if group.user_name != claim.user_name{
            return Err(AppError::UnauthorizedError)
        }

        // make sure this room does not exist before
        // get all the rooms
        let rooms = match GroupRepo::get_all_rooms_by_group_id(pool, req.group_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        for room in rooms{
            if room.name == req.name{
                return Err(AppError::AlreadyExistsError)
            }
        }
        let room = Room{
            id: Uuid::new_v4().to_string(),
            group_id:req.group_id.to_owned(),
            name: req.name.to_owned(),
            description: req.description.to_owned(),
            is_private: req.is_private,
            created_by: claim.user_name.clone(),
            code:None,
            member_count: 0,
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        GroupRepo::create_room(&pool, &room).await.map_err(|err| err)?;

        Ok(())
    }

    pub async fn join_room(
        pool:&PgPool,
        claim: &Claims,
        room_id:&String,
    )->Result<(), AppError>{
        // make sure room exists
        let room = match GroupRepo::get_room_by_id(pool, room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };

        // user can only join if the room is public
        if room.is_private{
            return Err(AppError::UnauthorizedError)
        }

        let room_user = UserRoom{
            user_name:claim.user_name.clone() ,
            room_id: room.id.clone(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        match GroupRepo::join_room(pool,&room_user ).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error joining room {}{}", room_user.user_name, err );
                return Err(err.into());
            }
        };
        Ok(())
    }

    pub async fn generate_code(
        pool:&PgPool,
        room_id:&String,
        claims: &Claims
    )->Result<String, AppError>{
        let mut room = match GroupRepo::get_room_by_id(pool, room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        debug!("gen by user {}", claims.user_name.clone());
        if claims.user_name.clone() != room.created_by{
            return Err(AppError::UnauthorizedError)
        }
        let code = thread_rng().sample_iter(&Alphanumeric).take(10).map(char::from).collect::<String>();
        room.code = Some(code.clone());
        match GroupRepo::update_room(&pool, &room).await{
            Ok(_)=>{},
            Err(err)=>{
                return Err(err);
            }
        }
        Ok(code)
    }

    pub async fn join_room_with_code(
        pool:&PgPool,
        claim: &Claims,
        code:Option<String>
    )->Result<(), AppError>{

        if code.is_none(){
            return Err(AppError::BadRequestError("code is missing".to_string()));
        }

        // make sure room exists
        let room = match GroupRepo::get_room_by_code(pool, code.clone().unwrap_or_default()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };

        if room.code.is_none(){
            return Err(AppError::BadRequestError("room has no code".to_string()));
        }

        if room.code.unwrap_or_default() != code.clone().unwrap() {
            return Err(AppError::UnauthorizedError);
        }


        let room_user = UserRoom{
            user_name:claim.user_name.clone() ,
            room_id: room.id.clone(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        match GroupRepo::join_room(pool,&room_user ).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error joining room {}{}", room_user.user_name, err );
                return Err(err.into());
            }
        };
        Ok(())
    }

    pub async fn update_group(
        pool:&PgPool,
        claim: &Claims,
        req:UpdateGroupReq
    )->Result<(),AppError>{
        // get group
        let mut group = match GroupRepo::get_group_by_id(pool, req.group_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        if group.user_name != claim.user_name{
            return Err(AppError::UnauthorizedError)
        }

        if req.name.is_some(){
            group.name = req.name.unwrap();
        }
        if req.description.is_some(){
            group.description = req.description;
        }
        if req.is_private.is_some(){
            group.is_private = req.is_private.unwrap();
        }


        group.updated_at = get_time_naive();
        match GroupRepo::update_group(pool, &group).await{
            Ok(_)=>{},
            Err(err)=>{
                return Err(err);
            }
        };
        Ok(())
    }

    pub async fn update_room(
        pool:&PgPool,
        claim: &Claims,
        req:UpdateRoomReq
    )->Result<(),AppError>{
        // get group
        let mut room = match GroupRepo::get_room_by_id(pool, req.room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        if room.created_by != claim.user_name{
            return Err(AppError::UnauthorizedError)
        }

        if req.name.is_some(){
            room.name = req.name.unwrap();
        }
        if req.description.is_some(){
            room.description = req.description;
        }
        if req.is_private.is_some(){
            room.is_private = req.is_private.unwrap();
        }


        room.updated_at = get_time_naive();
        match GroupRepo::update_room(pool, &room).await{
            Ok(_)=>{},
            Err(err)=>{
                return Err(err);
            }
        };
        Ok(())
    }

    pub async fn leave_room(
        pool:&PgPool,
        claim: &Claims,
        room_id:&String
    )->Result<(), AppError>{
        // make sure room exists
        let room = match GroupRepo::get_room_by_id(pool, room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };

        // Call the repository function to exit the room
        match GroupRepo::exit_room(pool, claim.user_name.clone(), room_id.clone()).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error leaving room {}{}", claim.user_name, err );
                return Err(err);
            }
        };
        Ok(())
    }

    pub async fn get_my_groups(
        pool:&PgPool,
        claim: &Claims
    )->Result<Vec<MyGroupsView>, AppError>{
        // Call the repository function to get the user's groups
        match GroupRepo::get_my_groups(pool, claim.user_name.clone()).await{
            Ok(groups) => Ok(groups),
            Err(err) => {
                error!("error getting groups for user {}: {}", claim.user_name, err);
                Err(err)
            }
        }
    }

    pub async fn get_group_by_id(
        pool:&PgPool,
        group_id: String
    )->Result<MyGroupsView, AppError>{
        // Call the repository function to get a single group with its rooms
        match GroupRepo::get_group_with_rooms_by_id(pool, group_id.clone()).await{
            Ok(group) => Ok(group),
            Err(err) => {
                error!("error getting group with id {}: {}", group_id, err);
                Err(err)
            }
        }
    }

    pub async fn get_room_with_members(
        pool: &PgPool,
        room_id: String
    ) -> Result<RoomWithMembersView, AppError> {
        // Call the repository function to get a single room with its members
        match GroupRepo::get_room_with_members(pool, room_id.clone()).await {
            Ok(room) => Ok(room),
            Err(err) => {
                error!("error getting room with id {}: {}", room_id, err);
                Err(err)
            }
        }
    }


    pub async fn check_if_user_can_connect(
        pool: &PgPool,
        user_name:String,
        room_id:String
    )->bool{
        let room = match GroupRepo::get_room_with_members(pool, room_id.clone()).await{
            Ok(room) => room,
            Err(err)=>{
                error!("error getting room {}", err);
                return false
            }
        };

        // chec if the user is a member if its a private group
        if !room.is_private{
           return true;
        }

        for member in room.members{
            if member.user_name == user_name{
                return true;
            }
        }

        // return false since user is not a member
        false
    }

    pub async fn group_chat_ws(
        mut session: Session,
        mut msg_stream: actix_ws::MessageStream,
        user_id: String,
        room_id: String,
        claim: &Claims,
        connections: web::Data<UserRoomSessions>,
        room_members: web::Data<RoomMembers>,
        pool: &PgPool
    ) -> Result<(), Error> {
        debug!("working on connections");

        if !Self::check_if_user_can_connect(pool,user_id.to_owned(), room_id.to_owned()).await{
            return Err(actix_web::error::ErrorUnauthorized("You cannot join this room"));
        }
        // add user session if its not already there
        connections.entry(user_id.clone()).or_insert(session.clone());
        // Register the user in room
        room_members
            .entry(room_id.to_string())
            .or_default()
            .entry(user_id.to_owned())
            .or_insert_with(|| {
                println!("User {} connected to room {}", user_id, room_id);
            });
        //println!("User {} connected to room {}", user_id, room_id.to_owned());

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    // Process incoming message
                    if let Ok(req) = serde_json::from_str::<RoomChatReq>(&text) {
                        println!("Received message: {:?}", req);
                        match req.message_type {
                            MessageType::RoomMessage => {


                                // create chat
                                let mut chat = RoomMessage{
                                    id: uuid::Uuid::new_v4().to_string(),
                                    image: None,
                                    text: req.message.to_owned(),
                                    created_at: get_time_naive(),
                                    updated_at: get_time_naive(),
                                    user_name: user_id.to_owned(),
                                    room_id:req.room_id,
                                };

                                if req.image.is_some(){
                                    chat.image = Some(req.image.clone().unwrap_or_default())
                                }


                                let res_chat =match GroupRepo::create_room_message(pool, chat.clone()).await{
                                    Ok(data)=>{
                                        data
                                    },
                                    Err(err)=>{
                                        log::error!("{}", err);
                                        return Err(actix_web::error::ErrorInternalServerError(""));
                                    }
                                };

                                let curr_room_members = if let Some(inner_map) = room_members.get(&room_id.to_owned()) {
                                    inner_map.iter().map(|entry| entry.key().clone()).collect()
                                } else {
                                    Vec::new()
                                };
                                debug!("sending message to room members ... {:?}", curr_room_members);
                                // Forward to each recipient if online
                                for user in curr_room_members {
                                    if user == user_id{
                                        // dont send to self
                                        continue;
                                    }
                                    if let Some(recipient_session) = connections.get(&user) {
                                        debug!("sending message to recipient session -- {}", recipient_session.key());
                                        let data_str =match  serde_json::to_string(&res_chat){
                                            Ok(d)=>{d},
                                            Err(err)=>{
                                                error!("{}", err);
                                                return Err(actix_web::error::ErrorInternalServerError("Error decoding string"));
                                            }
                                        };
                                        let mut session = recipient_session.value().clone();
                                        session.text(data_str).await.map_err(actix_web::error::ErrorInternalServerError)?;
                                        // Forward message to recipient
                                    } else {
                                        log::debug!("Recipient {} not online", user.to_owned());
                                        // send notififcation

                                        // get users profile
                                        let profile = match ProfileService::get_profile(pool, user.to_owned()).await{
                                            Ok(data)=>{data},
                                            Err(err)=>{
                                                log::error!("error getting profile {}", err.to_string());
                                                return Err(err.into())
                                            }
                                        };
                                        log::debug!("got profile :{}", profile.user_name.clone());
                                        // send notification if the user has a token
                                        let mut data_map = HashMap::new();
                                        data_map.insert("user_name".to_string(), profile.user_name.clone());
                                        if profile.app_f_token.is_some() {
                                            let payload = FcmMessage {
                                                message: MessagePayload {
                                                    token: profile.app_f_token.clone().unwrap(),
                                                    notification: Notification {
                                                        title: req.room_name.clone()+" in: "+&req.group_name.clone(),
                                                        body: truncate_string(req.message.clone()),

                                                    },
                                                    data: Some(data_map),
                                                },
                                            };

                                            match send_app_notification(payload).await{
                                                Ok(_)=>{
                                                    log::debug!("Successfully sent app notification");
                                                },
                                                Err(err)=>{
                                                    log::error!("error sending app notification {}", err.to_string());
                                                    return Err(err.into())
                                                }
                                            }
                                        }
                                    }
                                }

                            },
                            MessageType::JoinRoom => {
                                // add user to users sessions and room members in dhashmap
                                let groups = GroupRepo::get_my_groups()
                                // add user session if its not already there
                                connections.entry(user_id.clone()).or_insert(session.clone());
                                // Register the user in room
                                room_members
                                    .entry(req.room_id.to_string())
                                    .or_default()
                                    .entry(user_id.to_owned())
                                    .or_insert_with(|| {
                                        println!("User {} connected to room {}", user_id, room_id);
                                    });
                            }
                        }


                    }
                }
                Message::Ping(payload) => {
                    if session.pong(&payload).await.is_err() {
                        return Err(actix_web::error::ErrorInternalServerError(""));
                    }
                },
                Message::Close(reason) => {
                    println!("Connection closed for user {}: {:?}", user_id, reason);
                    break;
                }
                _ => (),
            }
        }

        // remove user room connection
        connections.remove(&user_id);
        // Remove user from active connections
        room_members.entry(room_id.clone()).and_modify(|room| {
            room.remove(&user_id);
            if room.is_empty() {
                room_members.remove(&room_id);
            }
        });
        println!("User {} disconnected from {}", user_id.to_owned(), room_id.to_owned());
        Ok(())
    }
}
