use actix_web::{
    get, post,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use actix_web_validator::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::notifications::models::{BroadcastReq, BroadcastResp, MarkReadReq, NotificationItem, RegisterTokenReq, UnreadCountResp};
use crate::notifications::service::NotificationService;
use crate::shared::auth::Claims;
use crate::shared::response::GenericResp;

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[get("/list")]
pub async fn list_notifications(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
    query: Query<ListQuery>,
) -> HttpResponse {
    let mut resp = GenericResp::<Vec<NotificationItem>> {
        message: "".to_string(),
        server_message: None,
        data: None,
    };
    let claim = match claim {
        Some(c) => c,
        None => {
            resp.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(resp);
        }
    };
    match NotificationService::list(
        &pool,
        &claim.user_name,
        query.limit.unwrap_or(20),
        query.offset.unwrap_or(0),
    )
    .await
    {
        Ok(items) => {
            resp.data = Some(items);
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            log::error!("error listing notifications {}", err);
            resp.message = "Error getting notifications".to_string();
            resp.server_message = Some(err.to_string());
            HttpResponse::BadRequest().json(resp)
        }
    }
}

#[get("/unread_count")]
pub async fn unread_count(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut resp = GenericResp::<UnreadCountResp> {
        message: "".to_string(),
        server_message: None,
        data: None,
    };
    let claim = match claim {
        Some(c) => c,
        None => {
            resp.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(resp);
        }
    };
    match NotificationService::unread_count(&pool, &claim.user_name).await {
        Ok(count) => {
            resp.data = Some(count);
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            log::error!("error getting unread count {}", err);
            resp.message = "Error getting unread count".to_string();
            resp.server_message = Some(err.to_string());
            HttpResponse::BadRequest().json(resp)
        }
    }
}

#[post("/read")]
pub async fn mark_read(
    pool: Data<PgPool>,
    req: Json<MarkReadReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut resp = GenericResp::<String> {
        message: "".to_string(),
        server_message: None,
        data: None,
    };
    let claim = match claim {
        Some(c) => c,
        None => {
            resp.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(resp);
        }
    };
    match NotificationService::mark_read(&pool, &claim.user_name, req.ids.as_deref()).await {
        Ok(_) => {
            resp.message = "OK".to_string();
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            log::error!("error marking notifications read {}", err);
            resp.message = "Error updating notifications".to_string();
            resp.server_message = Some(err.to_string());
            HttpResponse::BadRequest().json(resp)
        }
    }
}

#[derive(Debug, Serialize)]
struct TokenResp {
    saved: bool,
}
#[post("/register_token")]
pub async fn register_token(
    pool: Data<PgPool>,
    req: Json<RegisterTokenReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut resp = GenericResp::<TokenResp> {
        message: "".to_string(),
        server_message: None,
        data: None,
    };
    let claim = match claim {
        Some(c) => c,
        None => {
            resp.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(resp);
        }
    };
    if req.token.trim().is_empty() {
        resp.message = "token is required".to_string();
        return HttpResponse::BadRequest().json(resp);
    }
    match NotificationService::save_token(&pool, &claim.user_name, req.token.trim()).await {
        Ok(_) => {
            resp.message = "Token saved".to_string();
            resp.data = Some(TokenResp { saved: true });
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            log::error!("error saving push token {}", err);
            resp.message = "Error saving token".to_string();
            resp.server_message = Some(err.to_string());
            HttpResponse::BadRequest().json(resp)
        }
    }
}

/// Admin mass notification to every user (inbox row + push).
#[post("/broadcast")]
pub async fn admin_broadcast(
    pool: Data<PgPool>,
    req: Json<BroadcastReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut resp = GenericResp::<BroadcastResp> {
        message: "".to_string(),
        server_message: None,
        data: None,
    };
    let claim = match claim {
        Some(c) => c,
        None => {
            resp.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(resp);
        }
    };
    if claim.role != "ADMIN" {
        resp.message = "Admin only".to_string();
        return HttpResponse::Forbidden().json(resp);
    }
    match NotificationService::broadcast(&pool, &req.title, &req.body, req.link.as_deref()).await
    {
        Ok(result) => {
            resp.message = format!(
                "Broadcast sent to {} users ({} pushes)",
                result.users_notified, result.pushes_sent
            );
            resp.data = Some(result);
            HttpResponse::Ok().json(resp)
        }
        Err(err) => {
            log::error!("error sending broadcast {}", err);
            resp.message = "Error sending broadcast".to_string();
            resp.server_message = Some(err.to_string());
            HttpResponse::BadRequest().json(resp)
        }
    }
}
