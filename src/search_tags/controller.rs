use actix_web::{delete, get, post, put, web, HttpResponse};
use actix_web::web::{Data, ReqData};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::search_tags::models::{CreateTagReq, FriendSearchHit, MyTagsResp, SearchFriendsReq, SearchTag, SetMyTagsReq, UpdateTagReq};
use crate::search_tags::service::SearchTagsService;
use crate::shared::auth::Claims;
use crate::shared::error::AppError;
use crate::shared::response::GenericResp;

fn unauthorized<T: Serialize>() -> HttpResponse {
    HttpResponse::Unauthorized().json(GenericResp::<T> {
        message: "Unauthorized".to_string(),
        server_message: None,
        data: None,
    })
}

fn app_error<T: Serialize>(err: AppError, default_msg: &str) -> HttpResponse {
    let (mut status, message) = match &err {
        AppError::UnauthorizedError => (HttpResponse::Unauthorized(), "You are not authorized".to_string()),
        AppError::NotFoundError(_, _) => (HttpResponse::NotFound(), "Not found".to_string()),
        AppError::BadRequestError(data) => (HttpResponse::BadRequest(), data.clone()),
        other => {
            error!("search_tags error: {}", other);
            (HttpResponse::InternalServerError(), default_msg.to_string())
        }
    };
    status.json(GenericResp::<T> {
        message,
        server_message: None,
        data: None,
    })
}

// GET /search/tags?kind=skill|interest — everyone with auth can list
#[derive(Deserialize)]
struct ListQuery {
    kind: Option<String>,
}

#[get("/tags")]
pub async fn list_tags(pool: Data<PgPool>, query: web::Query<ListQuery>, claim: Option<ReqData<Claims>>) -> HttpResponse {
    let _claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<Vec<SearchTag>>(),
    };
    match SearchTagsService::list(&pool, query.kind.clone()).await {
        Ok(tags) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(tags),
        }),
        Err(err) => app_error::<Vec<SearchTag>>(err, "Error listing skills and interests"),
    }
}

// POST /search/friends — advanced friend search (text + skills + interests)
#[post("/friends")]
pub async fn search_friends(
    pool: Data<PgPool>,
    body: web::Json<SearchFriendsReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<Vec<FriendSearchHit>>(),
    };
    match SearchTagsService::search_friends(&pool, &claims, body.into_inner()).await {
        Ok(hits) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(hits),
        }),
        Err(err) => app_error::<Vec<FriendSearchHit>>(err, "Error searching friends"),
    }
}

// GET /profile/my_tags
#[get("/my_tags")]
pub async fn get_my_tags(pool: Data<PgPool>, claim: Option<ReqData<Claims>>) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<MyTagsResp>(),
    };
    match SearchTagsService::get_my_tags(&pool, &claims).await {
        Ok(resp) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(resp),
        }),
        Err(err) => app_error::<MyTagsResp>(err, "Error getting your skills and interests"),
    }
}

// POST /profile/my_tags
#[post("/my_tags")]
pub async fn set_my_tags(
    pool: Data<PgPool>,
    body: web::Json<SetMyTagsReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<MyTagsResp>(),
    };
    match SearchTagsService::set_my_tags(&pool, &claims, body.tag_ids.clone()).await {
        Ok(resp) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(resp),
        }),
        Err(err) => app_error::<MyTagsResp>(err, "Error saving your skills and interests"),
    }
}

// ---------- Admin ----------

// GET /admin/search/tags
#[get("/tags")]
pub async fn admin_list_tags(pool: Data<PgPool>, claim: Option<ReqData<Claims>>) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<Vec<SearchTag>>(),
    };
    // Service enforces ADMIN; pass a dummy kind=None list via list() after role check.
    match SearchTagsService::list(&pool, None).await {
        Ok(tags) => {
            if claims.role != "ADMIN" {
                return unauthorized::<Vec<SearchTag>>();
            }
            HttpResponse::Ok().json(GenericResp {
                message: "Ok".to_string(),
                server_message: None,
                data: Some(tags),
            })
        }
        Err(err) => app_error::<Vec<SearchTag>>(err, "Error listing tags"),
    }
}

// POST /admin/search/tags
#[post("/tags")]
pub async fn admin_create_tag(
    pool: Data<PgPool>,
    body: web::Json<CreateTagReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<SearchTag>(),
    };
    match SearchTagsService::create(&pool, &claims, body.name.clone(), body.kind.clone()).await {
        Ok(tag) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(tag),
        }),
        Err(err) => app_error::<SearchTag>(err, "Error creating tag"),
    }
}

// PUT /admin/search/tags/{id}
#[put("/tags/{id}")]
pub async fn admin_update_tag(
    pool: Data<PgPool>,
    path: web::Path<String>,
    body: web::Json<UpdateTagReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<SearchTag>(),
    };
    match SearchTagsService::update(&pool, &claims, path.into_inner(), body.name.clone(), body.kind.clone()).await {
        Ok(tag) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(tag),
        }),
        Err(err) => app_error::<SearchTag>(err, "Error updating tag"),
    }
}

// DELETE /admin/search/tags/{id}
#[delete("/tags/{id}")]
pub async fn admin_delete_tag(
    pool: Data<PgPool>,
    path: web::Path<String>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim {
        Some(c) => c.into_inner(),
        None => return unauthorized::<String>(),
    };
    match SearchTagsService::delete(&pool, &claims, path.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(GenericResp::<String> {
            message: "Deleted".to_string(),
            server_message: None,
            data: None,
        }),
        Err(err) => app_error::<String>(err, "Error deleting tag"),
    }
}
