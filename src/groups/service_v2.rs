use std::collections::HashMap;

use actix_web::{web, Error as ActixError};
use actix_ws::{Message, Session};
use futures_util::StreamExt;
use log::{debug, error};
use sqlx::PgPool;
use uuid::Uuid;

use crate::groups::models::{
    invite_code, CallParticipant, CallSessions, CreateGroupV2Req, GroupCall, GroupCallState,
    GroupClientFrame, GroupMessageView, GroupPresence, GroupServerFrame, GroupSessions,
    UpdateGroupV2Req,
};
use crate::groups::repository_v2::GroupV2Repo;
use crate::shared::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};
use crate::shared::auth::Claims;
use crate::shared::error::AppError;
use crate::shared::general::get_time_naive;
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
        // owner or group admin may edit info
        if g.user_name != claim.user_name && !GroupV2Repo::is_group_admin(pool, group_id, &claim.user_name).await.unwrap_or(false) {
            return Err(AppError::UnauthorizedError);
        }
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

    /// Owner-only: promote member -> admin, demote admin -> member.
    pub async fn set_role(
        pool: &PgPool,
        claim: &Claims,
        group_id: &str,
        target: &str,
        action: &str,
    ) -> Result<String, AppError> {
        let g = GroupV2Repo::get_group(pool, group_id).await?;
        if g.user_name != claim.user_name {
            return Err(AppError::UnauthorizedError);
        }
        if target == g.user_name {
            return Err(AppError::BadRequestError("Cannot change the owner role".to_string()));
        }
        // target must be a member
        let m = GroupV2Repo::is_member(pool, group_id, target).await?;
        if m.is_none() {
            return Err(AppError::NotFoundError("member".to_string(), target.to_string()));
        }
        match action {
            "promote" => {
                GroupV2Repo::set_member_role(pool, group_id, target, "admin").await?;
                Ok("admin".to_string())
            }
            "demote" => {
                GroupV2Repo::set_member_role(pool, group_id, target, "member").await?;
                Ok("member".to_string())
            }
            _ => Err(AppError::BadRequestError("action must be promote|demote".to_string())),
        }
    }

    // ---------- group calls (audio, single speaker, P2P signaling) ----------
    fn call_inactive(group_id: &str) -> GroupServerFrame {
        GroupServerFrame::CallState { call: GroupCall::inactive(group_id) }
    }

    async fn broadcast_call(
        sessions: &GroupSessions,
        presence: &GroupPresence,
        group_id: &str,
        call: Option<GroupCall>,
    ) {
        let frame = match call {
            Some(c) => GroupServerFrame::CallState { call: c.to_state() },
            None => Self::call_inactive(group_id),
        };
        let Ok(s) = serde_json::to_string(&frame) else { return };
        let online = Self::online_in(presence, group_id);
        for u in online {
            for mut sess in Self::sessions_for(sessions, &u) {
                let _ = sess.text(s.clone()).await;
            }
        }
    }

    async fn send_call_to(sessions: &GroupSessions, user: &str, frame: &GroupServerFrame) {
        if let Ok(s) = serde_json::to_string(frame) {
            for mut sess in Self::sessions_for(sessions, user) {
                let _ = sess.text(s.clone()).await;
            }
        }
    }

    /// Succession priority: group owner on call wins (handover on rejoin),
    /// else earliest-joined group admin on call, else earliest-joined participant.
    /// `roles`: user_name -> group role (owner/admin/member).
    fn elect_call_admin(
        participants: &[CallParticipant],
        group_owner: &str,
        roles: &std::collections::HashMap<String, String>,
    ) -> Option<String> {
        if participants.is_empty() {
            return None;
        }
        if participants.iter().any(|p| p.user_name == group_owner) {
            return Some(group_owner.to_string());
        }
        // earliest admin on the call
        let mut admins: Vec<&CallParticipant> = participants
            .iter()
            .filter(|p| roles.get(&p.user_name).map(|r| r == "owner" || r == "admin").unwrap_or(false))
            .collect();
        admins.sort_by(|a, b| a.joined_at.cmp(&b.joined_at));
        if let Some(a) = admins.first() {
            return Some(a.user_name.clone());
        }
        // fallback: earliest joiner
        let mut sorted = participants.to_vec();
        sorted.sort_by(|a, b| a.joined_at.cmp(&b.joined_at));
        sorted.first().map(|p| p.user_name.clone())
    }

    async fn roles_for(pool: &PgPool, group_id: &str, users: &[String]) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        for u in users {
            let role = GroupV2Repo::is_member(pool, group_id, u)
                .await
                .ok()
                .flatten()
                .map(|m| m.role)
                .unwrap_or_else(|| "member".to_string());
            map.insert(u.clone(), role);
        }
        // owner row may be stored on groups table only; ensure owner maps to owner
        if let Ok(g) = GroupV2Repo::get_group(pool, group_id).await {
            map.entry(g.user_name.clone()).or_insert("owner".to_string());
            // owner is always owner regardless of member row
            map.insert(g.user_name, "owner".to_string());
        }
        map
    }

    pub async fn call_state(pool: &PgPool, calls: &CallSessions, group_id: &str) -> GroupCallState {
        if let Some(c) = calls.get(group_id) {
            c.to_state()
        } else {
            GroupCall::inactive(group_id)
        }
    }

    pub async fn call_start(
        pool: &PgPool,
        calls: &CallSessions,
        sessions: &GroupSessions,
        presence: &GroupPresence,
        claim: &Claims,
        group_id: &str,
    ) -> Result<GroupCallState, AppError> {
        if !GroupV2Repo::is_group_admin(pool, group_id, &claim.user_name).await? {
            return Err(AppError::UnauthorizedError);
        }
        let g = GroupV2Repo::get_group(pool, group_id).await?;
        if let Some(c) = calls.get(group_id) {
            return Ok(c.to_state());
        }
        let now = get_time_naive();
        let role = GroupV2Repo::is_member(pool, group_id, &claim.user_name)
            .await?
            .map(|m| m.role)
            .unwrap_or_else(|| "admin".to_string());
        let call = GroupCall {
            group_id: group_id.to_string(),
            started_by: claim.user_name.clone(),
            started_at: now,
            version: 1,
            speaker: claim.user_name.clone(),
            call_admin: claim.user_name.clone(),
            group_owner: g.user_name,
            participants: vec![CallParticipant {
                user_name: claim.user_name.clone(),
                joined_at: now,
                group_role: role,
            }],
            hands: vec![],
        };
        let state = call.to_state();
        calls.insert(group_id.to_string(), call);
        Self::broadcast_call(sessions, presence, group_id, calls.get(group_id).map(|c| c.clone())).await;
        Ok(state)
    }

    pub async fn call_end(
        pool: &PgPool,
        calls: &CallSessions,
        sessions: &GroupSessions,
        presence: &GroupPresence,
        claim: &Claims,
        group_id: &str,
    ) -> Result<(), AppError> {
        let is_call_admin = calls.get(group_id).map(|c| c.call_admin == claim.user_name).unwrap_or(false);
        let is_group_admin = GroupV2Repo::is_group_admin(pool, group_id, &claim.user_name).await.unwrap_or(false);
        if !is_call_admin && !is_group_admin {
            return Err(AppError::UnauthorizedError);
        }
        calls.remove(group_id);
        Self::broadcast_call(sessions, presence, group_id, None).await;
        Ok(())
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

    async fn handle_call_leave(
        pool: &PgPool,
        calls: &CallSessions,
        sessions: &GroupSessions,
        presence: &GroupPresence,
        group_id: &str,
        user: &str,
    ) {
        let (became_empty, ended) = {
            let Some(mut call) = calls.get_mut(group_id) else { return };
            call.participants.retain(|p| p.user_name != user);
            call.hands.retain(|h| h != user);
            if call.participants.is_empty() {
                (true, false)
            } else {
                let mut changed = false;
                // speaker left -> mic falls back to call admin (or newly elected admin)
                if call.speaker == user {
                    call.speaker = call.call_admin.clone();
                    changed = true;
                }
                // call admin left -> elect successor by join order (owner > admin > earliest)
                if call.call_admin == user {
                    let users: Vec<String> = call.participants.iter().map(|p| p.user_name.clone()).collect();
                    // drop borrow of call before await: clone needed fields
                    let owner = call.group_owner.clone();
                    let parts = call.participants.clone();
                    drop(call);
                    let roles = Self::roles_for(pool, group_id, &users).await;
                    if let Some(mut call) = calls.get_mut(group_id) {
                        if let Some(next) = Self::elect_call_admin(&parts, &owner, &roles) {
                            call.call_admin = next.clone();
                            // if speaker was the leaver or dangling, point mic at new admin
                            if call.speaker == user || !call.participants.iter().any(|p| p.user_name == call.speaker) {
                                call.speaker = next;
                            }
                        }
                        call.version += 1;
                        changed = true;
                    }
                    let snapshot = calls.get(group_id).map(|c| c.clone());
                    Self::broadcast_call(sessions, presence, group_id, snapshot).await;
                    return;
                }
                // speaker dangling (removed elsewhere) -> reset to admin
                if !call.participants.iter().any(|p| p.user_name == call.speaker) {
                    call.speaker = call.call_admin.clone();
                    changed = true;
                }
                if changed {
                    call.version += 1;
                }
                (false, changed)
            }
        };
        if became_empty {
            calls.remove(group_id);
            Self::broadcast_call(sessions, presence, group_id, None).await;
        } else if ended {
            let snapshot = calls.get(group_id).map(|c| c.clone());
            Self::broadcast_call(sessions, presence, group_id, snapshot).await;
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
        calls: web::Data<CallSessions>,
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
                        GroupClientFrame::CallStart { group_id } => {
                            let claim = Claims { role: String::new(), email: String::new(), user_name: user.clone(), membership: false, exp: 0 };
                            match Self::call_start(pool, &calls, &sessions, &presence, &claim, &group_id).await {
                                Ok(_) => {}
                                Err(_) => {
                                    let ef = GroupServerFrame::Error { temp_id: None, message: "Cannot start call".to_string() };
                                    Self::send_call_to(&sessions, &user, &ef).await;
                                }
                            }
                        }
                        GroupClientFrame::CallJoin { group_id } => {
                            if GroupV2Repo::is_member(pool, &group_id, &user).await.unwrap_or(None).is_none() {
                                let ef = GroupServerFrame::Error { temp_id: None, message: "You are not a member".to_string() };
                                Self::send_call_to(&sessions, &user, &ef).await;
                                continue;
                            }
                            let Some(mut call) = calls.get_mut(&group_id) else {
                                let ef = GroupServerFrame::Error { temp_id: None, message: "No active call".to_string() };
                                Self::send_call_to(&sessions, &user, &ef).await;
                                continue;
                            };
                            if !call.participants.iter().any(|p| p.user_name == user) {
                                let role = GroupV2Repo::is_member(pool, &group_id, &user).await.ok().flatten().map(|m| m.role).unwrap_or_else(|| "member".to_string());
                                call.participants.push(CallParticipant { user_name: user.clone(), joined_at: get_time_naive(), group_role: role });
                            }
                            // handover on rejoin: owner > earliest admin > earliest joiner
                            let users: Vec<String> = call.participants.iter().map(|p| p.user_name.clone()).collect();
                            let owner = call.group_owner.clone();
                            let parts = call.participants.clone();
                            drop(call);
                            let roles = Self::roles_for(pool, &group_id, &users).await;
                            if let Some(mut call) = calls.get_mut(&group_id) {
                                if let Some(next) = Self::elect_call_admin(&parts, &owner, &roles) {
                                    call.call_admin = next;
                                }
                                call.version += 1;
                            }
                            let snapshot = calls.get(&group_id).map(|c| c.clone());
                            Self::broadcast_call(&sessions, &presence, &group_id, snapshot).await;
                        }
                        GroupClientFrame::CallLeave { group_id } => {
                            Self::handle_call_leave(pool, &calls, &sessions, &presence, &group_id, &user).await;
                        }
                        GroupClientFrame::CallEnd { group_id } => {
                            let claim = Claims { role: String::new(), email: String::new(), user_name: user.clone(), membership: false, exp: 0 };
                            let _ = Self::call_end(pool, &calls, &sessions, &presence, &claim, &group_id).await;
                        }
                        GroupClientFrame::RaiseHand { group_id } => {
                            if let Some(mut call) = calls.get_mut(&group_id) {
                                let in_call = call.participants.iter().any(|p| p.user_name == user);
                                if in_call && call.speaker != user && !call.hands.contains(&user) {
                                    call.hands.push(user.clone());
                                    call.version += 1;
                                    let snapshot = call.clone();
                                    drop(call);
                                    Self::broadcast_call(&sessions, &presence, &group_id, Some(snapshot)).await;
                                }
                            }
                        }
                        GroupClientFrame::LowerHand { group_id } => {
                            if let Some(mut call) = calls.get_mut(&group_id) {
                                if call.hands.contains(&user) {
                                    call.hands.retain(|h| h != &user);
                                    call.version += 1;
                                    let snapshot = call.clone();
                                    drop(call);
                                    Self::broadcast_call(&sessions, &presence, &group_id, Some(snapshot)).await;
                                }
                            }
                        }
                        GroupClientFrame::GrantMic { group_id, to } => {
                            let allowed = if let Some(call) = calls.get(&group_id) {
                                let from_ok = call.call_admin == user || call.speaker == user;
                                let to_in_call = call.participants.iter().any(|p| p.user_name == to);
                                // holder may only pass to someone raising hand; call admin may grant to anyone in call
                                let hands_ok = call.call_admin == user || call.hands.contains(&to);
                                from_ok && to_in_call && hands_ok
                            } else { false };
                            if allowed {
                                if let Some(mut call) = calls.get_mut(&group_id) {
                                    call.speaker = to.clone();
                                    call.hands.retain(|h| h != &to);
                                    call.version += 1;
                                    let snapshot = call.clone();
                                    drop(call);
                                    Self::broadcast_call(&sessions, &presence, &group_id, Some(snapshot)).await;
                                }
                            } else {
                                let ef = GroupServerFrame::Error { temp_id: None, message: "Cannot grant mic".to_string() };
                                Self::send_call_to(&sessions, &user, &ef).await;
                            }
                        }
                        GroupClientFrame::RevokeMic { group_id } => {
                            let allowed = calls.get(&group_id).map(|c| c.call_admin == user).unwrap_or(false);
                            if allowed {
                                if let Some(mut call) = calls.get_mut(&group_id) {
                                    call.speaker = call.call_admin.clone();
                                    call.version += 1;
                                    let snapshot = call.clone();
                                    drop(call);
                                    Self::broadcast_call(&sessions, &presence, &group_id, Some(snapshot)).await;
                                }
                            } else {
                                let ef = GroupServerFrame::Error { temp_id: None, message: "Only call admin can take mic".to_string() };
                                Self::send_call_to(&sessions, &user, &ef).await;
                            }
                        }
                        GroupClientFrame::CallOffer { group_id, to, sdp } => {
                            // only speaker broadcasts offers
                            let ok = calls.get(&group_id).map(|c| c.speaker == user && c.participants.iter().any(|p| p.user_name == to)).unwrap_or(false);
                            if ok {
                                let f = GroupServerFrame::CallOffer { group_id, from: user.clone(), sdp };
                                Self::send_call_to(&sessions, &to, &f).await;
                            }
                        }
                        GroupClientFrame::CallAnswer { group_id, to, sdp } => {
                            let ok = calls.get(&group_id).map(|c| c.participants.iter().any(|p| p.user_name == to) && c.participants.iter().any(|p| p.user_name == user)).unwrap_or(false);
                            if ok {
                                let f = GroupServerFrame::CallAnswer { group_id, from: user.clone(), sdp };
                                Self::send_call_to(&sessions, &to, &f).await;
                            }
                        }
                        GroupClientFrame::CallIce { group_id, to, candidate } => {
                            let ok = calls.get(&group_id).map(|c| c.participants.iter().any(|p| p.user_name == to) && c.participants.iter().any(|p| p.user_name == user)).unwrap_or(false);
                            if ok {
                                let f = GroupServerFrame::CallIce { group_id, from: user.clone(), candidate };
                                Self::send_call_to(&sessions, &to, &f).await;
                            }
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
            // leave all calls (triggers succession / mic fallback)
            let in_calls: Vec<String> = calls.iter().filter(|e| e.participants.iter().any(|p| p.user_name == user)).map(|e| e.group_id.clone()).collect();
            for gid in in_calls {
                Self::handle_call_leave(pool, &calls, &sessions, &presence, &gid, &user).await;
            }
        }
        debug!("group ws disconnected {}", user);
        Ok(())
    }
}
