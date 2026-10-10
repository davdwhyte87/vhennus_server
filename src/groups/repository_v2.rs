use chrono::NaiveDateTime;
use log::error;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::groups::models::{
    slug_category, DbGroup, GroupCategoryRow, GroupDetail, GroupJoinRequest, GroupMemberRow,
    GroupMessage, GroupMessageView, GroupReplyPreview, GroupTopic, GroupUnreadItem, JoinRequestView, MyGroupItem,
};
use crate::profile::models::MiniProfile;
use crate::shared::error::AppError;
use crate::shared::general::get_time_naive;

fn db_err(err: sqlx::Error) -> AppError {
    error!("db error: {}", err);
    if let Some(db) = err.as_database_error() {
        if db.code().as_deref() == Some("23505") {
            return AppError::AlreadyExistsError;
        }
    }
    AppError::DBInsertError
}

fn fetch_err(what: &str, err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::RowNotFound => {
            error!("{} not found: {}", what, err);
            AppError::NotFoundError(what.to_string(), "".to_string())
        }
        _ => {
            error!("fetch {} error: {}", what, err);
            AppError::FetchDataError
        }
    }
}

#[derive(sqlx::FromRow)]
struct MyGroupCursor {
    group_id: String,
    role: String,
    last_read_at: NaiveDateTime,
}

pub struct GroupV2Repo;

impl GroupV2Repo {
    // ---------- groups ----------
    pub async fn insert_group(
        pool: &PgPool,
        owner: &str,
        name: &str,
        about: Option<&str>,
        image: Option<&str>,
        is_private: bool,
        invite: &str,
        legacy_cats: &[String],
    ) -> Result<DbGroup, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = get_time_naive();
        sqlx::query(
            "INSERT INTO groups(id, user_name, name, description, about, is_private, image, invite_code, category, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
        )
        .bind(&id)
        .bind(owner)
        .bind(name)
        .bind(about)
        .bind(about)
        .bind(is_private)
        .bind(image)
        .bind(invite)
        .bind(legacy_cats)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(db_err)?;
        Self::get_group(pool, &id).await
    }

    pub async fn get_group(pool: &PgPool, id: &str) -> Result<DbGroup, AppError> {
        sqlx::query_as::<Postgres, DbGroup>(
            "SELECT id, user_name, name, about, description, is_private, image, invite_code, created_at, updated_at FROM groups WHERE id=$1",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| fetch_err("groups", e))
    }

    pub async fn get_group_by_invite(pool: &PgPool, code: &str) -> Result<DbGroup, AppError> {
        sqlx::query_as::<Postgres, DbGroup>(
            "SELECT id, user_name, name, about, description, is_private, image, invite_code, created_at, updated_at FROM groups WHERE invite_code=$1",
        )
        .bind(code)
        .fetch_one(pool)
        .await
        .map_err(|e| fetch_err("groups", e))
    }

    pub async fn update_group(
        pool: &PgPool,
        id: &str,
        name: Option<&str>,
        about: Option<&str>,
        image: Option<&str>,
        is_private: Option<bool>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE groups SET
               name = COALESCE($2, name),
               about = COALESCE($3, about),
               description = COALESCE($3, description),
               image = COALESCE($4, image),
               is_private = COALESCE($5, is_private),
               updated_at = NOW()
             WHERE id=$1",
        )
        .bind(id)
        .bind(name)
        .bind(about)
        .bind(image)
        .bind(is_private)
        .execute(pool)
        .await
        .map_err(|e| {
            error!("update group error: {}", e);
            AppError::DBUpdateError
        })?;
        Ok(())
    }

    pub async fn set_group_categories(
        pool: &PgPool,
        group_id: &str,
        categories: &[String],
    ) -> Result<(), AppError> {
        // ensure rows exist, then replace map
        for cat in categories {
            let cid = slug_category(cat);
            sqlx::query("INSERT INTO group_categories(id, name) VALUES ($1,$2) ON CONFLICT (id) DO NOTHING")
                .bind(&cid)
                .bind(cat.trim())
                .execute(pool)
                .await
                .map_err(|e| {
                    error!("ensure category error: {}", e);
                    AppError::DBInsertError
                })?;
        }
        sqlx::query("DELETE FROM group_category_map WHERE group_id=$1")
            .bind(group_id)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("clear categories error: {}", e);
                AppError::DBUpdateError
            })?;
        for cat in categories {
            let cid = slug_category(cat);
            // map by id, fallback to lookup by name for seeded rows with different ids
            let done = sqlx::query("INSERT INTO group_category_map(group_id, category_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
                .bind(group_id)
                .bind(&cid)
                .execute(pool)
                .await;
            if done.is_err() {
                continue;
            }
            // if the cid did not exist (seeded id differs), resolve by name
            let row: Option<(String,)> = sqlx::query_as("SELECT id FROM group_categories WHERE lower(name)=lower($1)")
                .bind(cat.trim())
                .fetch_optional(pool)
                .await
                .unwrap_or(None);
            if let Some((real_id,)) = row {
                if real_id != cid {
                    let _ = sqlx::query("INSERT INTO group_category_map(group_id, category_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
                        .bind(group_id)
                        .bind(&real_id)
                        .execute(pool)
                        .await;
                }
            }
        }
        // keep legacy groups.category (TEXT[] NOT NULL) in sync for old readers
        let _ = sqlx::query("UPDATE groups SET category=$2, updated_at=NOW() WHERE id=$1")
            .bind(group_id)
            .bind(categories)
            .execute(pool)
            .await;
        Ok(())
    }

    pub async fn group_categories(pool: &PgPool, group_id: &str) -> Result<Vec<String>, AppError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT c.name FROM group_category_map m JOIN group_categories c ON c.id=m.category_id WHERE m.group_id=$1 ORDER BY c.name")
                .bind(group_id)
                .fetch_all(pool)
                .await
                .map_err(|e| fetch_err("group_categories", e))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn member_count(pool: &PgPool, group_id: &str) -> Result<i64, AppError> {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM group_members WHERE group_id=$1")
            .bind(group_id)
            .fetch_one(pool)
            .await
            .map_err(|e| fetch_err("group_members", e))?;
        Ok(n)
    }

    // ---------- membership ----------
    pub async fn is_member(pool: &PgPool, group_id: &str, user: &str) -> Result<Option<GroupMemberRow>, AppError> {
        let row: Option<GroupMemberRow> = sqlx::query_as::<Postgres, GroupMemberRow>(
            "SELECT group_id, user_name, role, last_read_at, last_read_msg_id, created_at FROM group_members WHERE group_id=$1 AND user_name=$2",
        )
        .bind(group_id)
        .bind(user)
        .fetch_optional(pool)
        .await
        .map_err(|e| fetch_err("group_members", e))?;
        Ok(row)
    }

    pub async fn add_member(pool: &PgPool, group_id: &str, user: &str, role: &str) -> Result<(), AppError> {
        // NOTE: last_read_at is written from the app clock (not DB NOW()):
        // message stamps also come from the app clock, so unread comparisons
        // stay exact regardless of the DB session timezone.
        sqlx::query("INSERT INTO group_members(group_id, user_name, role, last_read_at) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING")
            .bind(group_id)
            .bind(user)
            .bind(role)
            .bind(get_time_naive())
            .execute(pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    pub async fn remove_member(pool: &PgPool, group_id: &str, user: &str) -> Result<bool, AppError> {
        let r = sqlx::query("DELETE FROM group_members WHERE group_id=$1 AND user_name=$2")
            .bind(group_id)
            .bind(user)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("remove member error: {}", e);
                AppError::DBDeleteError
            })?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn list_members(pool: &PgPool, group_id: &str) -> Result<Vec<(GroupMemberRow, Option<MiniProfile>)>, AppError> {
        let members: Vec<GroupMemberRow> = sqlx::query_as::<Postgres, GroupMemberRow>(
            "SELECT group_id, user_name, role, last_read_at, last_read_msg_id, created_at FROM group_members WHERE group_id=$1 ORDER BY CASE role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END, created_at",
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
        .map_err(|e| fetch_err("group_members", e))?;
        let mut out = Vec::with_capacity(members.len());
        for m in members {
            let p: Option<MiniProfile> = sqlx::query_as::<Postgres, MiniProfile>(
                "SELECT user_name, image, bio, name FROM profiles WHERE user_name=$1",
            )
            .bind(&m.user_name)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
            out.push((m, p));
        }
        Ok(out)
    }

    // ---------- search / my groups ----------
    pub async fn search_groups(
        pool: &PgPool,
        q: Option<&str>,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DbGroup>, AppError> {
        // Private groups are never searchable; members see them via /my or direct link.
        let rows: Vec<DbGroup> = sqlx::query_as::<Postgres, DbGroup>(
            "SELECT DISTINCT g.id, g.user_name, g.name, g.about, g.description, g.is_private, g.image, g.invite_code, g.created_at, g.updated_at
             FROM groups g
             LEFT JOIN group_category_map m ON m.group_id=g.id
             LEFT JOIN group_categories c ON c.id=m.category_id
             WHERE g.is_private=false
               AND ($1 IS NULL OR g.name ILIKE '%'||$1||'%' OR COALESCE(g.about,'') ILIKE '%'||$1||'%')
               AND ($2 IS NULL OR lower(c.name)=lower($2))
             ORDER BY g.created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(q.unwrap_or(""))
        .bind(category)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| fetch_err("groups", e))?;
        // sqlx binds NULL vs "" — treat empty q as no filter
        let _ = q;
        Ok(rows)
    }

    pub async fn my_group_ids(pool: &PgPool, user: &str) -> Result<Vec<(String, String, NaiveDateTime)>, AppError> {
        // (group_id, role, last_read_at)
        let rows: Vec<MyGroupCursor> = sqlx::query_as::<Postgres, MyGroupCursor>(
            "SELECT group_id, role, last_read_at FROM group_members WHERE user_name=$1 ORDER BY created_at DESC",
        )
        .bind(user)
        .fetch_all(pool)
        .await
        .map_err(|e| fetch_err("group_members", e))?;
        Ok(rows.into_iter().map(|r| (r.group_id, r.role, r.last_read_at)).collect())
    }

    pub async fn unread_for(pool: &PgPool, group_id: &str, since: &chrono::NaiveDateTime) -> Result<i64, AppError> {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM group_messages WHERE group_id=$1 AND created_at > $2")
            .bind(group_id)
            .bind(since)
            .fetch_one(pool)
            .await
            .map_err(|e| fetch_err("group_messages", e))?;
        Ok(n)
    }

    pub async fn open_topic(pool: &PgPool, group_id: &str) -> Result<Option<GroupTopic>, AppError> {
        let row: Option<GroupTopic> = sqlx::query_as::<Postgres, GroupTopic>(
            "SELECT id, group_id, title, is_open, created_by, created_at, closed_at FROM group_topics WHERE group_id=$1 AND is_open=true ORDER BY created_at DESC LIMIT 1",
        )
        .bind(group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| fetch_err("group_topics", e))?;
        Ok(row)
    }

    pub async fn last_message(pool: &PgPool, group_id: &str) -> Result<Option<GroupMessage>, AppError> {
        let row: Option<GroupMessage> = sqlx::query_as::<Postgres, GroupMessage>(
            "SELECT id, group_id, sender, text, image, topic_id, reply_to_msg_id, created_at FROM group_messages WHERE group_id=$1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| fetch_err("group_messages", e))?;
        Ok(row)
    }

    pub async fn hydrate_message(pool: &PgPool, msg: &GroupMessage) -> GroupMessageView {
        let prof: Option<MiniProfile> = sqlx::query_as::<Postgres, MiniProfile>(
            "SELECT user_name, image, bio, name FROM profiles WHERE user_name=$1",
        )
        .bind(&msg.sender)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
        // quoted reply preview (same group only, best-effort)
        let reply_to: Option<GroupReplyPreview> = match msg.reply_to_msg_id.as_deref() {
            Some(rid) => {
                let quoted: Option<GroupMessage> = sqlx::query_as::<Postgres, GroupMessage>(
                    "SELECT id, group_id, sender, text, image, topic_id, reply_to_msg_id, created_at FROM group_messages WHERE id=$1 AND group_id=$2",
                )
                .bind(rid)
                .bind(&msg.group_id)
                .fetch_optional(pool)
                .await
                .unwrap_or(None);
                match quoted {
                    Some(q) => {
                        let qprof: Option<MiniProfile> = sqlx::query_as::<Postgres, MiniProfile>(
                            "SELECT user_name, image, bio, name FROM profiles WHERE user_name=$1",
                        )
                        .bind(&q.sender)
                        .fetch_optional(pool)
                        .await
                        .unwrap_or(None);
                        Some(GroupReplyPreview {
                            id: q.id,
                            sender: q.sender,
                            sender_name: qprof.as_ref().and_then(|p| p.name.clone()),
                            text: q.text,
                        })
                    }
                    None => None,
                }
            }
            None => None,
        };
        GroupMessageView {
            id: msg.id.clone(),
            group_id: msg.group_id.clone(),
            sender: msg.sender.clone(),
            sender_name: prof.as_ref().and_then(|p| p.name.clone()),
            sender_image: prof.as_ref().and_then(|p| p.image.clone()),
            text: msg.text.clone(),
            image: msg.image.clone(),
            topic_id: msg.topic_id.clone(),
            reply_to,
            created_at: msg.created_at,
        }
    }

    // ---------- topics ----------
    pub async fn create_topic(pool: &PgPool, group_id: &str, title: &str, by: &str) -> Result<GroupTopic, AppError> {
        if Self::open_topic(pool, group_id).await?.is_some() {
            return Err(AppError::AlreadyExistsError);
        }
        let t = GroupTopic {
            id: Uuid::new_v4().to_string(),
            group_id: group_id.to_string(),
            title: title.trim().to_string(),
            is_open: true,
            created_by: by.to_string(),
            created_at: get_time_naive(),
            closed_at: None,
        };
        sqlx::query("INSERT INTO group_topics(id, group_id, title, is_open, created_by, created_at) VALUES ($1,$2,$3,true,$4,$5)")
            .bind(&t.id)
            .bind(&t.group_id)
            .bind(&t.title)
            .bind(&t.created_by)
            .bind(t.created_at)
            .execute(pool)
            .await
            .map_err(db_err)?;
        Ok(t)
    }

    pub async fn close_topic(pool: &PgPool, group_id: &str, topic_id: &str) -> Result<(), AppError> {
        let r = sqlx::query("UPDATE group_topics SET is_open=false, closed_at=NOW() WHERE id=$1 AND group_id=$2 AND is_open=true")
            .bind(topic_id)
            .bind(group_id)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("close topic error: {}", e);
                AppError::DBUpdateError
            })?;
        if r.rows_affected() == 0 {
            return Err(AppError::NotFoundError("topic".to_string(), topic_id.to_string()));
        }
        Ok(())
    }

    // ---------- messages ----------
    pub async fn create_message(
        pool: &PgPool,
        group_id: &str,
        sender: &str,
        text: &str,
        image: Option<&str>,
        reply_to_msg_id: Option<&str>,
    ) -> Result<GroupMessage, AppError> {
        let topic = Self::open_topic(pool, group_id).await?;
        // a reply must reference a message from the same group
        let reply_to: Option<String> = match reply_to_msg_id.map(str::trim).filter(|s| !s.is_empty()) {
            Some(rid) => {
                let quoted: Option<(String, String)> = sqlx::query_as(
                    "SELECT id, group_id FROM group_messages WHERE id=$1",
                )
                .bind(rid)
                .fetch_optional(pool)
                .await
                .map_err(|e| fetch_err("group_messages", e))?;
                match quoted {
                    Some((qid, qgid)) if qgid == group_id => Some(qid),
                    _ => return Err(AppError::BadRequestError("replied message not found".to_string())),
                }
            }
            None => None,
        };
        // single timestamp for the row and the sender cursor (no self-race)
        let now = get_time_naive();
        let msg = GroupMessage {
            id: Uuid::new_v4().to_string(),
            group_id: group_id.to_string(),
            sender: sender.to_string(),
            text: text.to_string(),
            image: image.map(|s| s.to_string()),
            topic_id: topic.map(|t| t.id),
            reply_to_msg_id: reply_to,
            created_at: now,
        };
        sqlx::query("INSERT INTO group_messages(id, group_id, sender, text, image, topic_id, reply_to_msg_id, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(&msg.id)
            .bind(&msg.group_id)
            .bind(&msg.sender)
            .bind(&msg.text)
            .bind(&msg.image)
            .bind(&msg.topic_id)
            .bind(&msg.reply_to_msg_id)
            .bind(msg.created_at)
            .execute(pool)
            .await
            .map_err(db_err)?;
        // sender has read their own message (app clock, see add_member note)
        let _ = sqlx::query("UPDATE group_members SET last_read_at=$4, last_read_msg_id=$3 WHERE group_id=$1 AND user_name=$2")
            .bind(group_id)
            .bind(sender)
            .bind(&msg.id)
            .bind(now)
            .execute(pool)
            .await;
        Ok(msg)
    }

    pub async fn list_messages(
        pool: &PgPool,
        group_id: &str,
        limit: i64,
        before: Option<chrono::NaiveDateTime>,
    ) -> Result<Vec<GroupMessage>, AppError> {
        let rows: Vec<GroupMessage> = if let Some(b) = before {
            sqlx::query_as::<Postgres, GroupMessage>(
                "SELECT id, group_id, sender, text, image, topic_id, reply_to_msg_id, created_at FROM group_messages WHERE group_id=$1 AND created_at < $2 ORDER BY created_at DESC LIMIT $3",
            )
            .bind(group_id)
            .bind(b)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(|e| fetch_err("group_messages", e))?
        } else {
            sqlx::query_as::<Postgres, GroupMessage>(
                "SELECT id, group_id, sender, text, image, topic_id, reply_to_msg_id, created_at FROM group_messages WHERE group_id=$1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(group_id)
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(|e| fetch_err("group_messages", e))?
        };
        Ok(rows)
    }

    pub async fn mark_read(
        pool: &PgPool,
        group_id: &str,
        user: &str,
        last_msg_id: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE group_members SET last_read_at=$4, last_read_msg_id=COALESCE($3, last_read_msg_id) WHERE group_id=$1 AND user_name=$2")
            .bind(group_id)
            .bind(user)
            .bind(last_msg_id)
            .bind(get_time_naive())
            .execute(pool)
            .await
            .map_err(|e| {
                error!("mark read error: {}", e);
                AppError::DBUpdateError
            })?;
        Ok(())
    }

    // ---------- join requests ----------
    pub async fn create_join_request(pool: &PgPool, group_id: &str, user: &str) -> Result<GroupJoinRequest, AppError> {
        let id = Uuid::new_v4().to_string();
        // upsert to pending (re-request after reject allowed)
        sqlx::query(
            "INSERT INTO group_join_requests(id, group_id, user_name, status) VALUES ($1,$2,$3,'pending')
             ON CONFLICT (group_id, user_name) DO UPDATE SET status='pending', created_at=NOW(), id=EXCLUDED.id RETURNING id",
        )
        .bind(&id)
        .bind(group_id)
        .bind(user)
        .execute(pool)
        .await
        .map_err(db_err)?;
        let row: GroupJoinRequest = sqlx::query_as::<Postgres, GroupJoinRequest>(
            "SELECT id, group_id, user_name, status, created_at FROM group_join_requests WHERE group_id=$1 AND user_name=$2",
        )
        .bind(group_id)
        .bind(user)
        .fetch_one(pool)
        .await
        .map_err(|e| fetch_err("group_join_requests", e))?;
        Ok(row)
    }

    pub async fn list_requests(pool: &PgPool, group_id: &str, status: &str) -> Result<Vec<GroupJoinRequest>, AppError> {
        let rows: Vec<GroupJoinRequest> = sqlx::query_as::<Postgres, GroupJoinRequest>(
            "SELECT id, group_id, user_name, status, created_at FROM group_join_requests WHERE group_id=$1 AND status=$2 ORDER BY created_at DESC",
        )
        .bind(group_id)
        .bind(status)
        .fetch_all(pool)
        .await
        .map_err(|e| fetch_err("group_join_requests", e))?;
        Ok(rows)
    }

    pub async fn resolve_request(pool: &PgPool, group_id: &str, req_id: &str, accept: bool) -> Result<GroupJoinRequest, AppError> {
        let req: GroupJoinRequest = sqlx::query_as::<Postgres, GroupJoinRequest>(
            "SELECT id, group_id, user_name, status, created_at FROM group_join_requests WHERE id=$1 AND group_id=$2",
        )
        .bind(req_id)
        .bind(group_id)
        .fetch_one(pool)
        .await
        .map_err(|e| fetch_err("group_join_requests", e))?;
        if req.status != "pending" {
            return Err(AppError::BadRequestError("request already resolved".to_string()));
        }
        let status = if accept { "accepted" } else { "rejected" };
        sqlx::query("UPDATE group_join_requests SET status=$1 WHERE id=$2")
            .bind(status)
            .bind(req_id)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("resolve request error: {}", e);
                AppError::DBUpdateError
            })?;
        if accept {
            Self::add_member(pool, group_id, &req.user_name, "member").await?;
        }
        Ok(GroupJoinRequest { status: status.to_string(), ..req })
    }

    // ---------- admin ----------
    pub async fn admin_list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<DbGroup>, AppError> {
        let rows: Vec<DbGroup> = sqlx::query_as::<Postgres, DbGroup>(
            "SELECT id, user_name, name, about, description, is_private, image, invite_code, created_at, updated_at FROM groups ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| fetch_err("groups", e))?;
        Ok(rows)
    }

    pub async fn all_categories(pool: &PgPool) -> Result<Vec<GroupCategoryRow>, AppError> {
        let rows: Vec<GroupCategoryRow> = sqlx::query_as::<Postgres, GroupCategoryRow>(
            "SELECT id, name, created_at, updated_at FROM group_categories ORDER BY name",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| fetch_err("group_categories", e))?;
        Ok(rows)
    }

    pub async fn add_category(pool: &PgPool, name: &str) -> Result<GroupCategoryRow, AppError> {
        let id = slug_category(name);
        sqlx::query("INSERT INTO group_categories(id, name) VALUES ($1,$2) ON CONFLICT (id) DO NOTHING")
            .bind(&id)
            .bind(name.trim())
            .execute(pool)
            .await
            .map_err(db_err)?;
        let row: GroupCategoryRow = sqlx::query_as::<Postgres, GroupCategoryRow>(
            "SELECT id, name, created_at, updated_at FROM group_categories WHERE id=$1",
        )
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| fetch_err("group_categories", e))?;
        Ok(row)
    }

    pub async fn delete_category(pool: &PgPool, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM group_category_map WHERE category_id=$1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("delete category map error: {}", e);
                AppError::DBDeleteError
            })?;
        let r = sqlx::query("DELETE FROM group_categories WHERE id=$1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("delete category error: {}", e);
                AppError::DBDeleteError
            })?;
        if r.rows_affected() == 0 {
            return Err(AppError::NotFoundError("category".to_string(), id.to_string()));
        }
        Ok(())
    }

    // ---------- view builders ----------
    pub async fn to_detail(
        pool: &PgPool,
        g: &DbGroup,
        viewer: Option<&str>,
    ) -> Result<GroupDetail, AppError> {
        let cats = Self::group_categories(pool, &g.id).await.unwrap_or_default();
        let count = Self::member_count(pool, &g.id).await.unwrap_or(0);
        let topic = Self::open_topic(pool, &g.id).await.unwrap_or(None);
        let (is_member, my_role) = if let Some(u) = viewer {
            match Self::is_member(pool, &g.id, u).await? {
                Some(m) => (true, Some(m.role)),
                None => (false, None),
            }
        } else {
            (false, None)
        };
        let about = g.about.clone().or_else(|| g.description.clone());
        // invite code visible to members + owner; hidden from outsiders of private groups
        let invite = if is_member || g.user_name == viewer.unwrap_or("") {
            g.invite_code.clone()
        } else if !g.is_private {
            g.invite_code.clone()
        } else {
            None
        };
        Ok(GroupDetail {
            id: g.id.clone(),
            name: g.name.clone(),
            about,
            image: g.image.clone(),
            is_private: g.is_private,
            owner: g.user_name.clone(),
            invite_code: invite,
            categories: cats,
            member_count: count,
            is_member,
            my_role,
            open_topic: topic,
            created_at: g.created_at,
            updated_at: g.updated_at,
        })
    }

    pub async fn to_my_item(
        pool: &PgPool,
        g: &DbGroup,
        role: &str,
        last_read: &chrono::NaiveDateTime,
    ) -> Result<MyGroupItem, AppError> {
        let cats = Self::group_categories(pool, &g.id).await.unwrap_or_default();
        let count = Self::member_count(pool, &g.id).await.unwrap_or(0);
        let topic = Self::open_topic(pool, &g.id).await.unwrap_or(None);
        let unread = Self::unread_for(pool, &g.id, last_read).await.unwrap_or(0);
        let last = Self::last_message(pool, &g.id).await.unwrap_or(None);
        let last_view = match last {
            Some(m) => Some(Self::hydrate_message(pool, &m).await),
            None => None,
        };
        Ok(MyGroupItem {
            id: g.id.clone(),
            name: g.name.clone(),
            about: g.about.clone().or_else(|| g.description.clone()),
            image: g.image.clone(),
            is_private: g.is_private,
            owner: g.user_name.clone(),
            categories: cats,
            member_count: count,
            unread_count: unread,
            my_role: Some(role.to_string()),
            open_topic: topic,
            last_message: last_view,
            created_at: g.created_at,
            updated_at: g.updated_at,
        })
    }

    pub async fn unread_summary(pool: &PgPool, user: &str) -> Result<(i64, Vec<GroupUnreadItem>), AppError> {
        let ids = Self::my_group_ids(pool, user).await?;
        let mut items = Vec::new();
        for (gid, _, last_read) in ids {
            let n = Self::unread_for(pool, &gid, &last_read).await.unwrap_or(0);
            if n > 0 {
                items.push(GroupUnreadItem { group_id: gid, unread: n });
            }
        }
        let total = items.len() as i64;
        Ok((total, items))
    }

    pub async fn hydrate_request(pool: &PgPool, r: &GroupJoinRequest) -> JoinRequestView {
        let p: Option<MiniProfile> = sqlx::query_as::<Postgres, MiniProfile>(
            "SELECT user_name, image, bio, name FROM profiles WHERE user_name=$1",
        )
        .bind(&r.user_name)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
        JoinRequestView {
            id: r.id.clone(),
            group_id: r.group_id.clone(),
            user_name: r.user_name.clone(),
            status: r.status.clone(),
            created_at: r.created_at,
            profile: p,
        }
    }
}
