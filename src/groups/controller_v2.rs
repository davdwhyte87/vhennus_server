use std::collections::HashMap;

use actix_web::{delete, get, patch, post, web, Error, HttpRequest, HttpResponse};
use actix_web::web::{Data, Path, Query, ReqData};
use actix_ws::handle;
use log::error;
use serde::Deserialize;
use sqlx::PgPool;

use crate::groups::models::{
    AdminCategoryReq, CreateGroupV2Req, CreateTopicReq, GroupCategoryRow, GroupDetail,
    GroupMessageView, GroupPresence, GroupSessions, GroupUnreadResp, JoinRequestView, MarkReadReq,
    MyGroupItem, RespondJoinReq, UpdateGroupV2Req,
};
use crate::groups::repository_v2::GroupV2Repo;
use crate::groups::service_v2::GroupV2Service;
use crate::shared::auth::{decode_token, Claims};
use crate::shared::error::AppError;
use crate::shared::response::GenericResp;

fn err_status(err: AppError, default_msg: &str) -> HttpResponse {
    let (msg, mut status) = match &err {
        AppError::UnauthorizedError => ("You are not authorized".to_string(), HttpResponse::Unauthorized()),
        AppError::NotFoundError(_, _) => ("Not found".to_string(), HttpResponse::NotFound()),
        AppError::AlreadyExistsError => ("Already exists".to_string(), HttpResponse::Conflict()),
        AppError::BadRequestError(m) => (m.clone(), HttpResponse::BadRequest()),
        _ => {
            error!("group v2 error: {}", err);
            (default_msg.to_string(), HttpResponse::InternalServerError())
        }
    };
    status.json(GenericResp::<serde_json::Value> { message: msg, server_message: None, data: None })
}

fn ok<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(GenericResp { message: "Ok".to_string(), server_message: None, data: Some(data) })
}

fn require_admin(claim: &Claims) -> Result<(), HttpResponse> {
    if claim.role != "ADMIN" {
        return Err(HttpResponse::Forbidden().json(GenericResp::<()> {
            message: "Admin only".to_string(),
            server_message: None,
            data: None,
        }));
    }
    Ok(())
}

// POST /group/create
#[post("/create")]
pub async fn create_group_v2(pool: Data<PgPool>, body: web::Json<CreateGroupV2Req>, claim: ReqData<Claims>) -> HttpResponse {
    let claims = claim.into_inner();
    match GroupV2Service::create(&pool, &claims, &body.into_inner()).await {
        Ok(id) => ok(serde_json::json!({ "id": id })),
        Err(e) => err_status(e, "Error creating group"),
    }
}

// PATCH /group/{id}
#[patch("/{id}")]
pub async fn update_group_v2(
    pool: Data<PgPool>,
    path: Path<String>,
    body: web::Json<UpdateGroupV2Req>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    match GroupV2Service::update(&pool, &claim, &path.into_inner(), &body.into_inner()).await {
        Ok(_) => ok(serde_json::json!({ "ok": true })),
        Err(e) => err_status(e, "Error updating group"),
    }
}

// GET /group/my
#[get("/my")]
pub async fn my_groups_v2(pool: Data<PgPool>, claim: ReqData<Claims>) -> HttpResponse {
    let ids = match GroupV2Repo::my_group_ids(&pool, &claim.user_name).await {
        Ok(v) => v,
        Err(e) => return err_status(e, "Error fetching groups"),
    };
    let mut out: Vec<MyGroupItem> = Vec::with_capacity(ids.len());
    for (gid, role, last_read) in ids {
        let g = match GroupV2Repo::get_group(&pool, &gid).await {
            Ok(g) => g,
            Err(_) => continue,
        };
        if let Ok(item) = GroupV2Repo::to_my_item(&pool, &g, &role, &last_read).await {
            out.push(item);
        }
    }
    // most recent activity first (last message, else created)
    out.sort_by(|a, b| {
        let ta = a.last_message.as_ref().map(|m| m.created_at).unwrap_or(a.updated_at);
        let tb = b.last_message.as_ref().map(|m| m.created_at).unwrap_or(b.updated_at);
        tb.cmp(&ta)
    });
    ok(out)
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// GET /group/search?q=&category=&limit&offset  (public only)
#[get("/search")]
pub async fn search_groups_v2(pool: Data<PgPool>, query: Query<SearchQuery>, claim: ReqData<Claims>) -> HttpResponse {
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    let offset = query.offset.unwrap_or(0).max(0);
    let q = query.q.as_deref().filter(|s| !s.trim().is_empty());
    let cat = query.category.as_deref().filter(|s| !s.trim().is_empty());
    match GroupV2Repo::search_groups(&pool, q, cat, limit, offset).await {
        Ok(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for g in rows {
                if let Ok(d) = GroupV2Repo::to_detail(&pool, &g, Some(&claim.user_name)).await {
                    out.push(d);
                }
            }
            ok(out)
        }
        Err(e) => err_status(e, "Error searching groups"),
    }
}

// GET /group/invite/{code} — public preview for invite links
#[get("/invite/{code}")]
pub async fn invite_preview_v2(pool: Data<PgPool>, path: Path<String>, claim: ReqData<Claims>) -> HttpResponse {
    match GroupV2Repo::get_group_by_invite(&pool, &path.into_inner()).await {
        Ok(g) => match GroupV2Repo::to_detail(&pool, &g, Some(&claim.user_name)).await {
            Ok(d) => ok(d),
            Err(e) => err_status(e, "Error fetching group"),
        },
        Err(e) => err_status(e, "Invalid invite link"),
    }
}

// GET /group/{id}
#[get("/{id}")]
pub async fn get_group_v2(pool: Data<PgPool>, path: Path<String>, claim: ReqData<Claims>) -> HttpResponse {
    let id = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Error fetching group"),
    };
    if g.is_private {
        let member = GroupV2Repo::is_member(&pool, &id, &claim.user_name).await.unwrap_or(None);
        if member.is_none() && g.user_name != claim.user_name && claim.role != "ADMIN" {
            return err_status(AppError::UnauthorizedError, "Private group");
        }
    }
    match GroupV2Repo::to_detail(&pool, &g, Some(&claim.user_name)).await {
        Ok(d) => ok(d),
        Err(e) => err_status(e, "Error fetching group"),
    }
}

// GET /group/{id}/members
#[get("/{id}/members")]
pub async fn group_members_v2(pool: Data<PgPool>, path: Path<String>, claim: ReqData<Claims>) -> HttpResponse {
    let id = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    if g.is_private && GroupV2Repo::is_member(&pool, &id, &claim.user_name).await.unwrap_or(None).is_none() && claim.role != "ADMIN" {
        return err_status(AppError::UnauthorizedError, "Private group");
    }
    match GroupV2Repo::list_members(&pool, &id).await {
        Ok(rows) => {
            let out: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(m, p)| serde_json::json!({ "user_name": m.user_name, "role": m.role, "joined_at": m.created_at, "profile": p }))
                .collect();
            ok(out)
        }
        Err(e) => err_status(e, "Error fetching members"),
    }
}

// DELETE /group/{id}/members/{user} — owner only
#[delete("/{id}/members/{user}")]
pub async fn remove_member_v2(pool: Data<PgPool>, path: Path<(String, String)>, claim: ReqData<Claims>) -> HttpResponse {
    let (id, user) = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    if g.user_name != claim.user_name && claim.role != "ADMIN" {
        return err_status(AppError::UnauthorizedError, "Owner only");
    }
    if user == g.user_name {
        return err_status(AppError::BadRequestError("Cannot remove the owner".to_string()), "Cannot remove owner");
    }
    match GroupV2Repo::remove_member(&pool, &id, &user).await {
        Ok(true) => ok(serde_json::json!({ "ok": true })),
        Ok(false) => err_status(AppError::NotFoundError("member".to_string(), user), "Not a member"),
        Err(e) => err_status(e, "Error removing member"),
    }
}

// POST /group/{id}/join — public: join; private: request
#[post("/{id}/join")]
pub async fn join_group_v2(pool: Data<PgPool>, path: Path<String>, claim: ReqData<Claims>) -> HttpResponse {
    match GroupV2Service::join(&pool, &claim, &path.into_inner()).await {
        Ok(status) => ok(serde_json::json!({ "status": status })),
        Err(e) => err_status(e, "Error joining group"),
    }
}

// GET /group/{id}/requests?status=pending — owner views join requests + profiles
#[get("/{id}/requests")]
pub async fn list_requests_v2(
    pool: Data<PgPool>,
    path: Path<String>,
    query: Query<HashMap<String, String>>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    let id = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    if g.user_name != claim.user_name && claim.role != "ADMIN" {
        return err_status(AppError::UnauthorizedError, "Owner only");
    }
    let status = query.get("status").map(|s| s.as_str()).unwrap_or("pending");
    match GroupV2Repo::list_requests(&pool, &id, status).await {
        Ok(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(GroupV2Repo::hydrate_request(&pool, &r).await);
            }
            ok(out)
        }
        Err(e) => err_status(e, "Error fetching requests"),
    }
}

// POST /group/{id}/requests/{req_id} {action: accept|reject} — owner
#[post("/{id}/requests/{req_id}")]
pub async fn respond_request_v2(
    pool: Data<PgPool>,
    path: Path<(String, String)>,
    body: web::Json<RespondJoinReq>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    let (id, req_id) = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    if g.user_name != claim.user_name && claim.role != "ADMIN" {
        return err_status(AppError::UnauthorizedError, "Owner only");
    }
    let accept = match body.action.as_str() {
        "accept" => true,
        "reject" => false,
        _ => return err_status(AppError::BadRequestError("action must be accept|reject".to_string()), "Bad action"),
    };
    match GroupV2Repo::resolve_request(&pool, &id, &req_id, accept).await {
        Ok(r) => ok(serde_json::json!({ "status": r.status })),
        Err(e) => err_status(e, "Error resolving request"),
    }
}

// POST /group/{id}/topics {title} — owner adds topic (one open at a time)
#[post("/{id}/topics")]
pub async fn add_topic_v2(
    pool: Data<PgPool>,
    path: Path<String>,
    body: web::Json<CreateTopicReq>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    let id = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    if g.user_name != claim.user_name && claim.role != "ADMIN" {
        return err_status(AppError::UnauthorizedError, "Owner only");
    }
    if body.title.trim().is_empty() {
        return err_status(AppError::BadRequestError("topic title required".to_string()), "Bad topic");
    }
    match GroupV2Repo::create_topic(&pool, &id, &body.title, &claim.user_name).await {
        Ok(t) => ok(t),
        Err(AppError::AlreadyExistsError) => err_status(
            AppError::BadRequestError("Close the current topic first".to_string()),
            "Topic open",
        ),
        Err(e) => err_status(e, "Error adding topic"),
    }
}

// POST /group/{id}/topics/{tid}/close — owner
#[post("/{id}/topics/{tid}/close")]
pub async fn close_topic_v2(pool: Data<PgPool>, path: Path<(String, String)>, claim: ReqData<Claims>) -> HttpResponse {
    let (id, tid) = path.into_inner();
    let g = match GroupV2Repo::get_group(&pool, &id).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    if g.user_name != claim.user_name && claim.role != "ADMIN" {
        return err_status(AppError::UnauthorizedError, "Owner only");
    }
    match GroupV2Repo::close_topic(&pool, &id, &tid).await {
        Ok(_) => ok(serde_json::json!({ "ok": true })),
        Err(e) => err_status(e, "Error closing topic"),
    }
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub limit: Option<i64>,
    /// RFC3339 or "YYYY-MM-DD HH:MM:SS" cursor for pagination (exclusive)
    pub before: Option<String>,
}

// GET /group/{id}/messages?limit&before= — member only, newest-first
#[get("/{id}/messages")]
pub async fn group_messages_v2(
    pool: Data<PgPool>,
    path: Path<String>,
    query: Query<MessagesQuery>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    let id = path.into_inner();
    if GroupV2Repo::is_member(&pool, &id, &claim.user_name).await.unwrap_or(None).is_none() {
        return err_status(AppError::UnauthorizedError, "Join the group to view chats");
    }
    let limit = query.limit.unwrap_or(30).clamp(1, 100);
    let before = query.before.as_deref().and_then(|s| {
        if let Ok(d) = chrono::DateTime::parse_from_rfc3339(s) {
            Some(d.naive_local())
        } else {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()
        }
    });
    match GroupV2Repo::list_messages(&pool, &id, limit, before).await {
        Ok(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for m in rows {
                out.push(GroupV2Repo::hydrate_message(&pool, &m).await);
            }
            ok(out)
        }
        Err(e) => err_status(e, "Error fetching messages"),
    }
}

// POST /group/{id}/read {last_msg_id?} — REST fallback for read receipts
#[post("/{id}/read")]
pub async fn mark_read_v2(
    pool: Data<PgPool>,
    path: Path<String>,
    body: web::Json<MarkReadReq>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    let id = path.into_inner();
    match GroupV2Repo::mark_read(&pool, &id, &claim.user_name, body.last_msg_id.as_deref()).await {
        Ok(_) => ok(serde_json::json!({ "ok": true })),
        Err(e) => err_status(e, "Error marking read"),
    }
}

// GET /group/unread — badge: number of groups with unread
#[get("/unread")]
pub async fn unread_v2(pool: Data<PgPool>, claim: ReqData<Claims>) -> HttpResponse {
    match GroupV2Repo::unread_summary(&pool, &claim.user_name).await {
        Ok((total, groups)) => ok(GroupUnreadResp { total, groups }),
        Err(e) => err_status(e, "Error fetching unread"),
    }
}

// GET /group/categories
#[get("/categories")]
pub async fn list_categories_v2(pool: Data<PgPool>, _claim: ReqData<Claims>) -> HttpResponse {
    match GroupV2Repo::all_categories(&pool).await {
        Ok(rows) => ok(rows),
        Err(e) => err_status(e, "Error fetching categories"),
    }
}

// ---- admin ----
// GET /group/admin/groups?limit&offset
#[get("/admin/groups")]
pub async fn admin_list_groups_v2(
    pool: Data<PgPool>,
    query: Query<HashMap<String, String>>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    if let Err(r) = require_admin(&claim) {
        return r;
    }
    let limit: i64 = query.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20).clamp(1, 100);
    let offset: i64 = query.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0).max(0);
    match GroupV2Repo::admin_list(&pool, limit, offset).await {
        Ok(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for g in rows {
                let count = GroupV2Repo::member_count(&pool, &g.id).await.unwrap_or(0);
                let cats = GroupV2Repo::group_categories(&pool, &g.id).await.unwrap_or_default();
                out.push(serde_json::json!({
                    "id": g.id, "name": g.name,
                    "about": g.about.clone().or(g.description.clone()),
                    "image": g.image, "is_private": g.is_private, "owner": g.user_name,
                    "member_count": count, "categories": cats,
                    "invite_code": g.invite_code, "created_at": g.created_at,
                }));
            }
            ok(out)
        }
        Err(e) => err_status(e, "Error listing groups"),
    }
}

// GET /group/admin/groups/{id} — name, info, image, member count
#[get("/admin/groups/{id}")]
pub async fn admin_get_group_v2(pool: Data<PgPool>, path: Path<String>, claim: ReqData<Claims>) -> HttpResponse {
    if let Err(r) = require_admin(&claim) {
        return r;
    }
    let g = match GroupV2Repo::get_group(&pool, &path.into_inner()).await {
        Ok(g) => g,
        Err(e) => return err_status(e, "Group not found"),
    };
    match GroupV2Repo::to_detail(&pool, &g, Some(&claim.user_name)).await {
        Ok(d) => ok(d),
        Err(e) => err_status(e, "Error fetching group"),
    }
}

// POST /group/admin/categories {name}
#[post("/admin/categories")]
pub async fn admin_add_category_v2(
    pool: Data<PgPool>,
    body: web::Json<AdminCategoryReq>,
    claim: ReqData<Claims>,
) -> HttpResponse {
    if let Err(r) = require_admin(&claim) {
        return r;
    }
    if body.name.trim().is_empty() {
        return err_status(AppError::BadRequestError("name required".to_string()), "Bad category");
    }
    match GroupV2Repo::add_category(&pool, &body.name).await {
        Ok(c) => ok(c),
        Err(e) => err_status(e, "Error adding category"),
    }
}

// DELETE /group/admin/categories/{id}
#[delete("/admin/categories/{id}")]
pub async fn admin_delete_category_v2(pool: Data<PgPool>, path: Path<String>, claim: ReqData<Claims>) -> HttpResponse {
    if let Err(r) = require_admin(&claim) {
        return r;
    }
    match GroupV2Repo::delete_category(&pool, &path.into_inner()).await {
        Ok(_) => ok(serde_json::json!({ "ok": true })),
        Err(e) => err_status(e, "Error deleting category"),
    }
}

// ---- WS ----
// Browser WebSockets cannot set Authorization headers, so unlike the legacy
// /ws_group (AuthM + ReqData) this endpoint authenticates via ?token=
// and lives outside the auth scope (see main.rs), mirroring /chat/ws.
#[derive(Deserialize)]
pub struct GroupWsParams {
    pub token: String,
}

pub async fn ws_group_v2_connect(
    req: HttpRequest,
    stream: web::Payload,
    sessions: web::Data<GroupSessions>,
    presence: web::Data<GroupPresence>,
    query: web::Query<GroupWsParams>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let claims = decode_token(query.token.clone())
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))?;
    let (response, session, msg_stream) = handle(&req, stream)?;
    let user = claims.user_name.clone();
    actix_web::rt::spawn(async move {
        let _ = GroupV2Service::group_ws_v2(session, msg_stream, user, sessions, presence, &pool).await;
    });
    Ok(response)
}
