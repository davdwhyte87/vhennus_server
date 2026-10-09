use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse, Responder};
use actix_ws::{Message, Session};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::chat::models::{
    Chat, ChatPair, ChatPairView, CreateChatReq, PairUnread, UnreadResp,
};
use crate::chat::repository::{ChatPairRepo, ChatRepo};
use crate::profile::service::ProfileService;
use crate::user::service::FriendRequestService;
use crate::shared::app_notify::{
    send_app_notification, FcmMessage, MessagePayload, Notification,
};
use crate::shared::error::ServiceError;
use crate::shared::general::{get_current_time_stamp, get_time_naive};
use crate::shared::strings::truncate_string;

pub type UserConnections = Arc<DashMap<String, Session>>;

// Multi-session registry for 1:1 chat: every tab/device holds its own
// connection so live frames reach all of them (the legacy single-session
// map is still used by groups).
pub type ChatSessions = Arc<Mutex<HashMap<String, HashMap<Uuid, Session>>>>;

fn sessions_for(sessions: &ChatSessions, user: &str) -> Vec<Session> {
    sessions
        .lock()
        .map(|map| {
            map.get(user)
                .map(|conns| conns.values().cloned().collect())
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

async fn push_unread(pool: &PgPool, sessions: &ChatSessions, user: &str) {
    let snapshot = match ChatPairService::get_unread(pool, user.to_string()).await {
        Ok(s) => s,
        Err(err) => {
            log::error!("error computing unread for {}: {}", user, err);
            return;
        }
    };
    let frame = serde_json::json!({
        "type": "unread",
        "total": snapshot.total,
        "pairs": snapshot.pairs,
    })
    .to_string();
    for mut s in sessions_for(sessions, user) {
        let _ = s.text(frame.clone()).await;
    }
}

// Frames a client may send over the chat socket.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientFrame {
    Send {
        #[serde(default)]
        temp_id: Option<String>,
        receiver: String,
        message: Option<String>,
        image: Option<String>,
    },
    Read {
        pair_id: String,
    },
}

pub struct ChatService {}

impl ChatService {
    pub async fn create_chat(pool: &PgPool, chat: Chat) -> Result<Chat, Box<dyn Error>> {
        // both users must exist (user_exists returns bool — enforce it)
        if !ProfileService::user_exists(pool, &*chat.sender.clone()).await? {
            return Err("Sender does not exist".into());
        }
        if !ProfileService::user_exists(pool, &*chat.receiver.clone()).await? {
            return Err("Receiver does not exist".into());
        }
        if chat.sender == chat.receiver {
            return Err("You cannot chat with yourself".into());
        }
        // only friends can message each other
        if !FriendRequestService::are_friends(
            pool,
            chat.sender.clone(),
            chat.receiver.clone(),
        )
        .await?
        {
            return Err("You must be friends to chat".into());
        }

        // get chat pair
        let chat_pair = sqlx::query_as!(
            ChatPair,
            "
            SELECT * FROM chat_pairs
            WHERE (user1 = $1 AND user2 = $2) OR (user1 = $2 AND user2 = $1)
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
            // create chat pair and then chat (race-safe: unique index +
            // re-fetch on conflict)
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
            match ChatPairRepo::create_chat_pair(pool, &chat_pair).await {
                Ok(_) => {}
                Err(err) => {
                    // Possibly lost a race with another first message: if a
                    // pair now exists, use it; otherwise surface the error.
                    let existing = sqlx::query_as!(
                        ChatPair,
                        "SELECT * FROM chat_pairs
                         WHERE (user1 = $1 AND user2 = $2) OR (user1 = $2 AND user2 = $1)",
                        chat.sender.clone(),
                        chat.receiver.clone()
                    )
                    .fetch_optional(pool)
                    .await?;
                    match existing {
                        Some(row) => chat_pair_id = row.id,
                        None => return Err(err),
                    }
                }
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

    pub async fn find_pair_by_id(
        pool: &PgPool,
        id: String,
    ) -> Result<ChatPair, Box<dyn Error>> {
        ChatPairRepo::find_chat_pair_by_id(pool, id).await
    }

    pub async fn mark_read(
        pool: &PgPool,
        user_name: String,
        pair_id: String,
    ) -> Result<(), Box<dyn Error>> {
        let pair = ChatPairRepo::find_chat_pair_by_id(pool, pair_id.clone()).await?;
        if pair.user1 != user_name && pair.user2 != user_name {
            return Err("You are not a member of this chat".into());
        }
        ChatPairRepo::mark_pair_read(pool, pair_id, user_name, get_time_naive()).await
    }

    pub async fn get_unread(
        pool: &PgPool,
        user_name: String,
    ) -> Result<UnreadResp, Box<dyn Error>> {
        let rows = ChatPairRepo::get_unread(pool, user_name).await?;
        let mut pairs = Vec::new();
        for row in rows {
            let unread = row.unread.unwrap_or(0);
            if unread > 0 {
                pairs.push(PairUnread {
                    pair_id: row.pair_id,
                    unread,
                });
            }
        }
        // Badge counts unread conversations, not messages.
        let total = pairs.len() as i64;
        Ok(UnreadResp { total, pairs })
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
    sessions: web::Data<ChatSessions>,
    pool: &PgPool,
) -> Result<(), ActixError> {
    // Register this tab/device alongside any others the user has open.
    let conn_id = Uuid::new_v4();
    if let Ok(mut map) = sessions.lock() {
        map.entry(user_id.clone())
            .or_default()
            .insert(conn_id, session.clone());
    }
    println!("User {} connected ({})", user_id, conn_id);

    while let Some(Ok(msg)) = msg_stream.next().await {
        match msg {
            Message::Text(text) => {
                let frame: ClientFrame = match serde_json::from_str(&text) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                match frame {
                    ClientFrame::Send {
                        temp_id,
                        receiver,
                        message,
                        image,
                    } => {
                        let text_body = message.clone().unwrap_or_default();
                        if text_body.trim().is_empty() && image.is_none() {
                            let _ = session
                                .text(
                                    serde_json::json!({
                                        "type": "error",
                                        "temp_id": temp_id,
                                        "message": "Message is empty",
                                    })
                                    .to_string(),
                                )
                                .await;
                            continue;
                        }
                        let chat = Chat {
                            id: uuid::Uuid::new_v4().to_string(),
                            pair_id: "".to_string(),
                            sender: user_id.clone(),
                            receiver: receiver.clone(),
                            message: text_body,
                            image,
                            created_at: get_time_naive(),
                            updated_at: get_time_naive(),
                        };
                        let res_chat =
                            match ChatService::create_chat(pool, chat).await {
                                Ok(data) => data,
                                Err(err) => {
                                    // A bad message must not kill the connection.
                                    log::error!("error creating chat: {}", err);
                                    let _ = session
                                        .text(
                                            serde_json::json!({
                                                "type": "error",
                                                "temp_id": temp_id,
                                                "message": err.to_string(),
                                            })
                                            .to_string(),
                                        )
                                        .await;
                                    continue;
                                }
                            };
                        // Ack the sending tab so it can resolve its optimistic row.
                        let _ = session
                            .text(
                                serde_json::json!({
                                    "type": "sent",
                                    "temp_id": temp_id,
                                    "chat": res_chat,
                                })
                                .to_string(),
                            )
                            .await;
                        let new_frame = serde_json::json!({
                            "type": "new",
                            "chat": res_chat,
                        })
                        .to_string();
                        // Fan out to the sender's other tabs and every recipient tab.
                        for mut s in sessions_for(&sessions, &user_id) {
                            let _ = s.text(new_frame.clone()).await;
                        }
                        for mut s in sessions_for(&sessions, &receiver) {
                            let _ = s.text(new_frame.clone()).await;
                        }
                        push_unread(pool, &sessions, &receiver).await;
                        if sessions_for(&sessions, &receiver).is_empty() {
                            log::debug!("Recipient {} not online", receiver);
                            send_offline_notification(pool, &res_chat).await;
                        }
                    }
                    ClientFrame::Read { pair_id } => {
                        match ChatPairService::mark_read(
                            pool,
                            user_id.clone(),
                            pair_id,
                        )
                        .await
                        {
                            Ok(_) => push_unread(pool, &sessions, &user_id).await,
                            Err(err) => {
                                log::error!("error marking read: {}", err);
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

    // Remove only this connection; the user stays online via other tabs.
    if let Ok(mut map) = sessions.lock() {
        if let Some(conns) = map.get_mut(&user_id) {
            conns.remove(&conn_id);
            if conns.is_empty() {
                map.remove(&user_id);
            }
        }
    }
    println!("User {} disconnected", user_id);

    Ok(())
}

async fn send_offline_notification(pool: &PgPool, res_chat: &Chat) {
    // get users profile
    let profile = match ProfileService::get_profile(pool, res_chat.receiver.clone()).await {
        Ok(data) => data,
        Err(err) => {
            log::error!("error getting profile {}", err.to_string());
            return;
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
            }
        }
    }
}
