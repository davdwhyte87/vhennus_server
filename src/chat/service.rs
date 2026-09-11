use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse, Responder};
use actix_ws::{Message, Session};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use sqlx::PgPool;
use uuid::Uuid;

use crate::chat::models::{
    Chat, ChatPair, ChatPairView, CreateChatReq,
};
use crate::chat::repository::{ChatPairRepo, ChatRepo};
use crate::profile::service::ProfileService;
use crate::shared::app_notify::{
    send_app_notification, FcmMessage, MessagePayload, Notification,
};
use crate::shared::error::ServiceError;
use crate::shared::general::{get_current_time_stamp, get_time_naive};
use crate::shared::strings::truncate_string;

pub type UserConnections = Arc<DashMap<String, Session>>;

pub struct ChatService {}

impl ChatService {
    pub async fn create_chat(pool: &PgPool, chat: Chat) -> Result<Chat, Box<dyn Error>> {
        let xpool = pool.clone();
        // check that the users exist
        ProfileService::user_exists(pool, &*chat.sender.clone()).await?;
        ProfileService::user_exists(pool, &*chat.receiver.clone()).await?;

        // get chat pair
        let chat_pair = sqlx::query_as!(
            ChatPair,
            "
            SELECT * FROM chat_pairs WHERE user1 = $1 AND user2 = $2 OR user1 = $2 AND user2 =$1
            ",
            chat.sender.clone(),
            chat.receiver.clone()
        )
        .fetch_optional(pool)
        .await?;
        let mut chat_pair_id = "".to_string();
        if chat_pair.is_some() {
            chat_pair_id = chat_pair.unwrap().id;
        } else {
            // create chat pair and then chat
            chat_pair_id = Uuid::new_v4().to_string();
            let chat_pair = ChatPair {
                id: chat_pair_id.clone(),
                user1: chat.sender.clone(),
                user2: chat.receiver.clone(),
                last_message: Option::from(chat.message.clone()),
                all_read: false,
                created_at: get_time_naive(),
                updated_at: get_time_naive(),
            };
            ChatPairRepo::create_chat_pair(pool, &chat_pair).await?;
            // construct new chat
            let mut chat = chat;
            chat.id = Uuid::new_v4().to_string();
            chat.pair_id = chat_pair_id;
            chat.created_at = get_time_naive();
            chat.updated_at = get_time_naive();

            let result = chat.clone();

            //create chat
            ChatRepo::create_chat(pool, &chat).await?;

            return Ok(result);
        }

        // construct new chat
        let mut chat = chat;
        chat.id = Uuid::new_v4().to_string();
        chat.pair_id = chat_pair_id.clone();
        chat.created_at = get_time_naive();
        chat.updated_at = get_time_naive();

        let result = chat.clone();

        //create chat
        ChatRepo::create_chat(pool, &chat).await?;

        // update pair
        ChatPairRepo::update_chat_pair(pool, chat_pair_id.clone(), result.message.clone())
            .await?;
        Ok(result)
    }

    pub async fn get_chats_by_pair_id(
        pool: &PgPool,
        id: String,
    ) -> Result<Vec<Chat>, Box<dyn Error>> {
        ChatRepo::get_chats_by_pair_id(pool, id).await
    }
}

pub struct ChatPairService {}

impl ChatPairService {
    pub async fn find_chat_pair(
        pool: &PgPool,
        xuser1: String,
        xuser2: String,
    ) -> Result<ChatPairView, Box<dyn Error>> {
        ChatPairRepo::find_chat_pair(pool, xuser1, xuser2).await
    }

    pub async fn get_all_my_chat_pairs(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<ChatPairView>, Box<dyn Error>> {
        ChatPairRepo::get_all_my_chat_pairs(pool, user_name).await
    }
}

pub struct CircleService {}

impl CircleService {
    // Circle service methods would go here when migrated from MongoDB to Postgres.
}

pub async fn ws_chat(
    req: HttpRequest,
    body: web::Payload,
    connections: web::Data<UserConnections>,
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
                Message::Text(msg) => {}
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
    pool: &PgPool,
) -> Result<(), ActixError> {
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
                    let mut chat = Chat {
                        id: uuid::Uuid::new_v4().to_string(),
                        pair_id: if req.pair_id.is_some() {
                            req.pair_id.clone().unwrap()
                        } else {
                            "".to_string()
                        },
                        sender: user_id.clone(),
                        receiver: req.receiver.clone(),
                        message: "".to_string(),
                        image: None,
                        created_at: get_time_naive(),
                        updated_at: get_time_naive(),
                    };
                    if req.message.is_some() {
                        chat.message = req.message.clone().unwrap_or_default()
                    }
                    if req.image.is_some() {
                        chat.image = Some(req.image.clone().unwrap_or_default())
                    }

                    let res_chat = match ChatService::create_chat(pool, chat.clone()).await {
                        Ok(data) => data,
                        Err(err) => {
                            log::error!("{}", err);
                            return Err(actix_web::error::ErrorInternalServerError(""));
                        }
                    };
                    // Forward to recipient if online
                    if let Some(mut recipient_session) = connections.get_mut(&req.receiver) {
                        let data_str = match serde_json::to_string(&res_chat) {
                            Ok(d) => d,
                            Err(err) => {
                                return Err(actix_web::error::ErrorInternalServerError(
                                    "Error decoding string",
                                ));
                            }
                        };
                        recipient_session
                            .text(data_str)
                            .await
                            .map_err(actix_web::error::ErrorInternalServerError)?;
                        // Forward message to recipient
                    } else {
                        log::debug!("Recipient {} not online", req.receiver);
                        // send notififcation

                        // get users profile
                        let profile =
                            match ProfileService::get_profile(pool, res_chat.receiver.clone()).await
                            {
                                Ok(data) => data,
                                Err(err) => {
                                    log::error!("error getting profile {}", err.to_string());
                                    return Err(err.into());
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
                                        title: res_chat.receiver.clone(),
                                        body: truncate_string(res_chat.message.clone()),
                                    },
                                    data: Some(data_map),
                                },
                            };

                            match send_app_notification(payload).await {
                                Ok(_) => {
                                    log::debug!("Successfully sent app notification");
                                }
                                Err(err) => {
                                    log::error!("error sending app notification {}", err.to_string());
                                    return Err(err.into());
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
            }
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
