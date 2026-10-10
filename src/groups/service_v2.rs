use std::collections::HashMap;

use actix_web::{web, Error as ActixError};
use actix_ws::{Message, Session};
use futures_util::StreamExt;
use log::{debug, error};
use sqlx::PgPool;
use uuid::Uuid;

use crate::groups::models::{
    invite_code, CreateGroupV2Req, GroupClientFrame, GroupMessageView, GroupPresence,
    GroupServerFrame, GroupSessions, UpdateGroupV2Req,
};
use crate::groups::repository_v2::GroupV2Repo;
use crate::shared::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};
use crate::shared::auth::Claims;
use crate::shared::error::AppError;
use crate::shared::strings::truncate_string;

pub struct GroupV2Service;

impl GroupV2Service {
    fn owner_of(owner: &str, claim: &Claims) -> Result<(), AppError> {
        if owner != claim.user_name {
            return Err(AppError::UnauthorizedError);
        }
        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        claim: &Claims,
        req: &CreateGroupV2Req,
    ) -> Result<String, AppError> {
        let name = req.name.trim();
        if name.is_empty() || name.len() > 100 {
            return Err(AppError::BadRequestError("group name must be 1-100 chars".to_string()));
        }
        // retry once on invite collision (unique constraint)
        let mut last_err = AppError::DBInsertError;
        for _ in 0..2 {
            let code = invite_code();
            match GroupV2Repo::insert_group(
                pool,
                &claim.user_name,
                name,
                req.about.as_deref(),
                req.image.as_deref(),
                req.is_private,
                &code,
                &req.categories,
            )
            .await
            {
                Ok(g) => {
                    GroupV2Repo::add_member(pool, &g.id, &claim.user_name, "owner").await?;
                    if !req.categories.is_empty() {
                        GroupV2Repo::set_group_categories(pool, &g.id, &req.categories).await?;
                    }
                    return Ok(g.id);
                }
                Err(AppError::AlreadyExistsError) => return Err(AppError::AlreadyExistsError),
                Err(e) => {
                    last_err = e;
                }
            }
        }
        Err(last_err)
    }

    pub async fn update(
        pool: &PgPool,
        claim: &Claims,
        group_id: &str,
        req: &UpdateGroupV2Req,
    ) -> Result<(), AppError> {
        let g = GroupV2Repo::get_group(pool, group_id).await?;
        Self::owner_of(&g.user_name, claim)?;
        if let Some(n) = req.name.as_deref() {
            if n.trim().is_empty() || n.len() > 100 {
                return Err(AppError::BadRequestError("group name must be 1-100 chars".to_string()));
            }
        }
        GroupV2Repo::update_group(
            pool,
            group_id,
            req.name.as_deref(),
            req.about.as_deref(),
            req.image.as_deref(),
            req.is_private,
        )
        .await?;
        if let Some(cats) = req.categories.as_ref() {
            GroupV2Repo::set_group_categories(pool, group_id, cats).await?;
        }
        Ok(())
    }

    /// Invite-link join. Public → direct member. Private → join request.
    /// Returns "joined" | "requested" | "already".
    pub async fn join(pool: &PgPool, claim: &Claims, group_id: &str) -> Result<String, AppError> {
        let g = GroupV2Repo::get_group(pool, group_id).await?;
        if GroupV2Repo::is_member(pool, group_id, &claim.user_name).await?.is_some() {
            return Ok("already".to_string());
        }
        if g.is_private {
            GroupV2Repo::create_join_request(pool, group_id, &claim.user_name).await?;
            Ok("requested".to_string())
        } else {
            GroupV2Repo::add_member(pool, group_id, &claim.user_name, "member").await?;
            Ok("joined".to_string())
        }
    }

    pub async fn send_message(
        pool: &PgPool,
        claim: &Claims,
        group_id: &str,
        text: Option<&str>,
        image: Option<&str>,
        reply_to_msg_id: Option<&str>,
    ) -> Result<GroupMessageView, AppError> {
        let member = GroupV2Repo::is_member(pool, group_id, &claim.user_name).await?;
        if member.is_none() {
            return Err(AppError::UnauthorizedError);
        }
        let body = text.unwrap_or("").trim();
        if body.is_empty() && image.unwrap_or("").trim().is_empty() {
            return Err(AppError::BadRequestError("message is empty".to_string()));
        }
        if body.len() > 4000 {
            return Err(AppError::BadRequestError("message too long (max 4000)".to_string()));
        }
        let msg = GroupV2Repo::create_message(pool, group_id, &claim.user_name, body, image, reply_to_msg_id).await?;
        Ok(GroupV2Repo::hydrate_message(pool, &msg).await)
    }

    // ---------- WS helpers ----------
    fn sessions_for(sessions: &GroupSessions, user: &str) -> Vec<Session> {
        sessions
            .get(user)
            .map(|m| m.iter().map(|e| e.value().clone()).collect())
            .unwrap_or_default()
    }

    fn online_in(presence: &GroupPresence, group_id: &str) -> Vec<String> {
        presence
            .get(group_id)
            .map(|m| m.iter().map(|e| e.key().clone()).collect())
            .unwrap_or_default()
    }

    async fn push_unread(pool: &PgPool, sessions: &GroupSessions, user: &str) {
        let (total, items) = GroupV2Repo::unread_summary(pool, user).await.unwrap_or((0, vec![]));
        let frame = GroupServerFrame::Unread { total, groups: items };
        if let Ok(s) = serde_json::to_string(&frame) {
            for mut sess in Self::sessions_for(sessions, user) {
                let _ = sess.text(s.clone()).await;
            }
        }
    }

    async fn send_offline_batch(pool: &PgPool, group_id: &str, msg: &GroupMessageView, online: &[String]) {
        // members minus online, single query for tokens
        #[derive(sqlx::FromRow)]
        struct Tok {
            user_name: String,
            app_f_token: Option<String>,
        }
        let rows: Vec<Tok> = sqlx::query_as::<sqlx::Postgres, Tok>(
            "SELECT p.user_name, p.app_f_token FROM group_members m JOIN profiles p ON p.user_name=m.user_name WHERE m.group_id=$1",
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for row in rows {
            if row.user_name == msg.sender {
                continue;
            }
            if online.iter().any(|u| u == &row.user_name) {
                continue;
            }
            if let Some(token) = row.app_f_token {
                let mut data = HashMap::new();
                data.insert("group_id".to_string(), group_id.to_string());
                data.insert("sender".to_string(), msg.sender.clone());
                let payload = FcmMessage {
                    message: MessagePayload {
                        token,
                        notification: Notification {
                            title: format!("{}: new message", msg.sender),
                            body: truncate_string(msg.text.clone()),
                        },
                        data: Some(data),
                    },
                };
                if let Err(e) = send_app_notification(payload).await {
                    error!("group fcm error: {}", e);
                }
            }
        }
    }

    /// Single multiplexed socket per device. Client sends Join per open group,
    /// Send for messages, Read for cursors. Membership is re-checked on every frame.
    pub async fn group_ws_v2(
        mut session: Session,
        mut stream: actix_ws::MessageStream,
        user: String,
        sessions: web::Data<GroupSessions>,
        presence: web::Data<GroupPresence>,
        pool: &PgPool,
    ) -> Result<(), ActixError> {
        let conn_id = Uuid::new_v4().to_string();
        sessions.entry(user.clone()).or_default().insert(conn_id.clone(), session.clone());
        // auto-join all my groups so background groups still push + badge
        let mut my_groups: Vec<String> = Vec::new();
        if let Ok(ids) = GroupV2Repo::my_group_ids(pool, &user).await {
            for (gid, _, _) in ids {
                presence.entry(gid.clone()).or_default().insert(user.clone(), ());
                my_groups.push(gid);
            }
        }
        let joined_frame = GroupServerFrame::Joined { groups: my_groups.clone() };
        if let Ok(s) = serde_json::to_string(&joined_frame) {
            let _ = session.text(s).await;
        }
        Self::push_unread(pool, &sessions, &user).await;
        debug!("group ws connected {} ({})", user, conn_id);

        // groups this connection explicitly joined (for precise cleanup)
        let mut conn_groups: Vec<String> = my_groups;

        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    let frame: GroupClientFrame = match serde_json::from_str(&text) {
                        Ok(f) => f,
                        Err(_) => continue,
                    };
                    match frame {
                        GroupClientFrame::Join { group_id } => {
                            // verify membership before adding to presence
                            match GroupV2Repo::is_member(pool, &group_id, &user).await {
                                Ok(Some(_)) => {
                                    presence.entry(group_id.clone()).or_default().insert(user.clone(), ());
                                    if !conn_groups.contains(&group_id) {
                                        conn_groups.push(group_id.clone());
                                    }
                                    let online = Self::online_in(&presence, &group_id);
                                    let pf = GroupServerFrame::Presence { group_id, online };
                                    if let Ok(s) = serde_json::to_string(&pf) {
                                        let _ = session.text(s).await;
                                    }
                                }
                                _ => {
                                    let ef = GroupServerFrame::Error {
                                        temp_id: None,
                                        message: "You are not a member of this group".to_string(),
                                    };
                                    if let Ok(s) = serde_json::to_string(&ef) {
                                        let _ = session.text(s).await;
                                    }
                                }
                            }
                        }
                        GroupClientFrame::Send { temp_id, group_id, message, image, reply_to_msg_id } => {
                            match Self::send_message(pool, &Claims {
                                role: String::new(),
                                email: String::new(),
                                user_name: user.clone(),
                                membership: false,
                                exp: 0,
                            }, &group_id, message.as_deref(), image.as_deref(), reply_to_msg_id.as_deref()).await {
                                Ok(view) => {
                                    // ack sender
                                    let ack = GroupServerFrame::Sent { temp_id: temp_id.clone(), message: view.clone() };
                                    if let Ok(s) = serde_json::to_string(&ack) {
                                        let _ = session.text(s).await;
                                    }
                                    // fan out to every session of every other online member
                                    let online = Self::online_in(&presence, &group_id);
                                    let new_frame = GroupServerFrame::New { message: view.clone() };
                                    if let Ok(s) = serde_json::to_string(&new_frame) {
                                        for u in online.iter() {
                                            if u == &user {
                                                // deliver to sender's OTHER devices too
                                                for mut sess in Self::sessions_for(&sessions, u) {
                                                    let _ = sess.text(s.clone()).await;
                                                }
                                                continue;
                                            }
                                            for mut sess in Self::sessions_for(&sessions, u) {
                                                let _ = sess.text(s.clone()).await;
                                            }
                                            // unread bump for recipients
                                            Self::push_unread(pool, &sessions, u).await;
                                        }
                                    }
                                    // offline push (batched, async, non-blocking)
                                    let pool_c = pool.clone();
                                    let online_c = online.clone();
                                    let view_c = view.clone();
                                    let gid_c = group_id.clone();
                                    actix_web::rt::spawn(async move {
                                        Self::send_offline_batch(&pool_c, &gid_c, &view_c, &online_c).await;
                                    });
                                }
                                Err(e) => {
                                    let ef = GroupServerFrame::Error {
                                        temp_id,
                                        message: match e {
                                            AppError::UnauthorizedError => "You are not a member of this group".to_string(),
                                            AppError::BadRequestError(m) => m,
                                            _ => "Failed to send message".to_string(),
                                        },
                                    };
                                    if let Ok(s) = serde_json::to_string(&ef) {
                                        let _ = session.text(s).await;
                                    }
                                }
                            }
                        }
                        GroupClientFrame::Read { group_id, msg_id } => {
                            let _ = GroupV2Repo::mark_read(pool, &group_id, &user, msg_id.as_deref()).await;
                            Self::push_unread(pool, &sessions, &user).await;
                        }
                    }
                }
                Message::Ping(p) => {
                    if session.pong(&p).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        // precise cleanup: only this conn_id
        if let Some(m) = sessions.get(&user) {
            m.remove(&conn_id);
            if m.is_empty() {
                drop(m);
                sessions.remove(&user);
            }
        }
        // remove user from presence only if no remaining sessions
        if !sessions.contains_key(&user) {
            for gid in conn_groups {
                if let Some(m) = presence.get(&gid) {
                    m.remove(&user);
                    let empty = m.is_empty();
                    drop(m);
                    if empty {
                        presence.remove(&gid);
                    }
                }
            }
        }
        debug!("group ws disconnected {}", user);
        Ok(())
    }
}
