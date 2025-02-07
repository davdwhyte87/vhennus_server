
use std::sync::Arc;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{Message, ProtocolError, Session};
use dashmap::DashMap;
use futures_util::{StreamExt, SinkExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};

use crate::{models::{chat::Chat, request_models::CreateChatReq}, services::chat_service::ChatService, utils::general::get_current_time_stamp};


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
    database: &Database
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
                
                    
                    let res_chat =match ChatService::create_chat(database, &mut chat).await{
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
                        println!("Recipient {} not online", req.receiver);
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