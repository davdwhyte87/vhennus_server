
use std::sync::Arc;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{Message, ProtocolError, Session};
use awc::Client;
use dashmap::DashMap;
use futures_util::{StreamExt, SinkExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::{models::{chat::Chat, request_models::CreateChatReq}, services::{app_notify::{send_app_notification, FcmMessage}, chat_service::ChatService, profile_service::ProfileService}, utils::general::get_current_time_stamp,};
use crate::services::app_notify::{MessagePayload, Notification};
use crate::utils::general::get_time_naive;
use crate::utils::strings_stuff::truncate_string;

pub type UserConnections = Arc<DashMap<String, Session>>;

pub async fn ws_chat(req: HttpRequest, body: web::Payload, 
    connections:web::Data<UserConnections>
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => {

                },
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

pub async fn chat_ws_service(
    mut session: Session,
    mut msg_stream: actix_ws::MessageStream,
    user_id: String,
    connections: web::Data<UserConnections>,
    pool: &PgPool
) -> Result<(), Error> {
    // Register the user
    connections.insert(user_id.clone(), session.clone());
    println!("User {} connected", user_id);

    while let Some(Ok(msg)) = msg_stream.next().await {
        match msg {
            Message::Text(text) => {
                // Process incoming message
                if let Ok(req) = serde_json::from_str::<CreateChatReq>(&text) {
                    println!("Received message: {:?}", req);
        
                    // create chat
                    let mut chat = Chat{
                    id: uuid::Uuid::new_v4().to_string(),
                    pair_id: if req.pair_id.is_some(){req.pair_id.clone().unwrap()} else{"".to_string()},
                    sender: user_id.clone(),
                    receiver: req.receiver.clone(),
                    message: "".to_string(),
                    image: None,
                    created_at: get_time_naive(),
                    updated_at: get_time_naive()
                    };
                    if req.message.is_some(){
                        chat.message = req.message.clone().unwrap_or_default()
                    }
                    if req.image.is_some(){
                        chat.image = Some(req.image.clone().unwrap_or_default())
                    }
                
                    
                    let res_chat =match ChatService::create_chat(pool, chat.clone()).await{
                        Ok(data)=>{
                            data
                        },
                        Err(err)=>{
                            log::error!("{}", err);
                            return Err(actix_web::error::ErrorInternalServerError(""));
                        }
                    };
                    // Forward to recipient if online
                    if let Some(mut recipient_session) = connections.get_mut(&req.receiver) {
                       
                        let data_str =match  serde_json::to_string(&res_chat){
                            Ok(d)=>{d},
                            Err(err)=>{
                                  return Err(actix_web::error::ErrorInternalServerError("Error decoding string"));  
                            }
                        };
                        recipient_session.text(data_str).await.map_err(actix_web::error::ErrorInternalServerError)?;
                         // Forward message to recipient
                    } else {
                        log::debug!("Recipient {} not online", req.receiver);
                        // send notififcation

                     

                        // get users profile 
                        let profile = match ProfileService::get_profile(pool, res_chat.receiver.clone()).await{
                            Ok(data)=>{data},
                            Err(err)=>{
                                log::error!("error getting profile {}", err.to_string());
                                return Err(err.into())
                            }
                        };
                        log::debug!("got profile :{}", profile.user_name.clone());
                        // send notification if the user has a token
                        if profile.app_f_token.is_some() {
                            let payload = FcmMessage {
                                message: MessagePayload {
                                    token: profile.app_f_token.clone().unwrap(),
                                    notification: Notification {
                                        title: res_chat.receiver.clone(),
                                        body: truncate_string(res_chat.message.clone()),
                                    },
                                    data: None,
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

    // Remove user from active connections
    connections.remove(&user_id);
    println!("User {} disconnected", user_id);

    Ok(())
}