use std::{env, fmt, str::FromStr};

use actix_web::{
    get, post,
    web::{self, Data, ReqData},
    HttpResponse, ResponseError,
};
use actix_web_validator::Json;
use bigdecimal::{num_bigint::BigInt, BigDecimal};
use gcp_auth::provider;
use log::error;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::feed::models::{Comment, CreateCommentReq, CreatePostReq, PollData, Post, PostFeedWithPoll, PostWithCommentsAndPoll, VotePollReq};
use crate::feed::service::PostService;
use crate::notifications::service::NotificationService;
use crate::orders::controller::escrow_to_user;
use crate::profile::service::ProfileService;
use crate::shared::auth::Claims;
use crate::shared::general::get_time_naive;
use crate::shared::response::GenericResp;

#[post("/create")]
pub async fn create_post(
    pool: Data<PgPool>,
    req: Result<web::Json<CreatePostReq>, actix_web::Error>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<Post> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let req = match req {
        Ok(data) => data,
        Err(err) => {
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
    };
    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // post model
    let post_id = Uuid::new_v4().to_string();
    let mut new_post = Post {
        id: post_id.clone(),
        text: req.text.to_owned(),
        image: None,
        created_at: get_time_naive(),
        updated_at: get_time_naive(),
        user_name: claim.user_name.to_owned(),
    };

    if req.image.is_some() {
        new_post.image = Some(req.image.to_owned().unwrap())
    }

    // Validate optional poll.
    let poll_options: Option<Vec<String>> = match &req.poll_options {
        None => None,
        Some(opts) => {
            let cleaned: Vec<String> =
                opts.iter().map(|o| o.trim().to_string()).filter(|o| !o.is_empty()).collect();
            if cleaned.len() < 2 {
                respData.message = "Poll needs at least 2 non-empty options".to_string();
                respData.server_message = None;
                respData.data = None;
                return HttpResponse::BadRequest().json(respData);
            }
            if cleaned.len() > 5 {
                respData.message = "Poll can have at most 5 options".to_string();
                respData.server_message = None;
                respData.data = None;
                return HttpResponse::BadRequest().json(respData);
            }
            if cleaned.iter().any(|o| o.len() > 280) {
                respData.message = "Each poll option must be 280 characters or fewer".to_string();
                respData.server_message = None;
                respData.data = None;
                return HttpResponse::BadRequest().json(respData);
            }
            Some(cleaned)
        }
    };

    match PostService::create_post_with_poll(&pool, new_post, poll_options.as_deref()).await {
        Ok(_) => {}
        Err(err) => {
            log::error!(" error creating post {}", err.to_string());
            respData.message = "Error creating post".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
    };

    // silently try to pay the user
    let profile = match ProfileService::get_profile(&pool, claim.user_name.clone()).await {
        Ok(profile) => Some(profile),
        Err(err) => {
            error!("error getting profile {}", err);
            None
        }
    };
    if profile.is_some() {
        let mut new_profile = profile.clone().unwrap();
        if new_profile.is_earnings_activated {
            let post_amount = BigDecimal::from_str("62").unwrap_or_default();
            new_profile.unclaimed_earnings = new_profile.unclaimed_earnings + post_amount;
            ProfileService::update_profile(&pool, new_profile).await;
        }
    }

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = None;
    return HttpResponse::Ok().json(respData);
}

#[get("/all")]
pub async fn get_all_posts(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<PostFeedWithPoll>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let posts = match PostService::get_all_post_with_polls(&pool, &claim.user_name).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting posts {}", err.to_string());
            respData.message = "Error getting posts".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json(respData);
}

#[get("/allmy")]
pub async fn get_my_posts(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<PostFeedWithPoll>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let posts = match PostService::get_all_my_posts_with_polls(&pool, claim.user_name.clone()).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting posts {}", err.to_string());
            respData.message = "Error getting posts".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json(respData);
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct GetUsersPosts {
    user_name: String,
}
#[get("/all/{user_name}")]
pub async fn get_users_posts(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
    path: web::Path<GetUsersPosts>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<PostFeedWithPoll>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let posts = match PostService::get_user_posts_with_polls(&pool, path.user_name.clone(), claim.user_name.clone()).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting posts {}", err.to_string());
            respData.message = "Error getting posts".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json(respData);
}

#[derive(Deserialize)]
struct GetSinglePostPath {
    id: String,
}

#[post("/{id}/comment/create")]
pub async fn create_comment(
    pool: Data<PgPool>,
    req: Json<CreateCommentReq>,
    claim: Option<ReqData<Claims>>,
    path: web::Path<GetSinglePostPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<Comment> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: Some(Comment::default()),
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // post model
    let new_comment = Comment {
        id: Uuid::new_v4().to_string(),
        text: req.text.to_owned(),
        created_at: get_time_naive(),
        user_name: claim.user_name.to_owned(),
        post_id: path.id.to_owned(),
    };

    match PostService::create_comment(&pool, new_comment.clone()).await {
        Ok(_) => {}
        Err(err) => {
            log::error!(" error creating comment {}", err.to_string());
            respData.message = "Error creating comment".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
    };

    // Notify the post owner (best-effort, never blocks the response).
    {
        let pool_clone = pool.get_ref().clone();
        let actor = claim.user_name.clone();
        let post_id = path.id.clone();
        let preview: String = new_comment.text.chars().take(120).collect();
        actix_web::rt::spawn(async move {
            match PostService::get_post_owner(&pool_clone, &post_id).await {
                Ok(Some(owner)) if owner != actor => {
                    NotificationService::push(
                        &pool_clone,
                        &owner,
                        "comment",
                        "New comment on your post",
                        &format!("@{} commented: {}", actor, preview),
                        Some(&format!("/home/post/{}", post_id)),
                    )
                    .await;
                }
                _ => {}
            }
        });
    }

    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(new_comment);
    return HttpResponse::Ok().json(respData);
}

#[get("/single/{id}")]
pub async fn get_single_posts(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
    path: web::Path<GetSinglePostPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<PostWithCommentsAndPoll> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let posts = match PostService::get_single_post_with_polls(&pool, path.id.to_owned(), &claim.user_name).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting post {}", err.to_string());
            respData.message = "Error getting post".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };
    respData.message = "".to_string();
    respData.server_message = None;
    respData.data = Some(posts);
    return HttpResponse::Ok().json(respData);
}

#[post("/like/{id}")]
pub async fn like_post(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
    path: web::Path<GetSinglePostPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<Post> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let liked = match PostService::toggle_like(&pool, path.id.clone(), claim.user_name.clone()).await {
        Ok(liked) => liked,
        Err(err) => {
            log::error!("error updating post {}", err.to_string());
            respData.message = "Error saving data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::NotFound().json(respData);
        }
    };
    if liked {
        let pool_clone = pool.get_ref().clone();
        let actor = claim.user_name.clone();
        let post_id = path.id.clone();
        actix_web::rt::spawn(async move {
            match PostService::get_post_owner(&pool_clone, &post_id).await {
                Ok(Some(owner)) if owner != actor => {
                    NotificationService::push(
                        &pool_clone,
                        &owner,
                        "like",
                        "Someone liked your post",
                        &format!("@{} liked your post", actor),
                        Some(&format!("/home/post/{}", post_id)),
                    )
                    .await;
                }
                _ => {}
            }
        });
    }
    respData.message = "OK".to_string();
    respData.server_message = None;
    respData.data = None;
    return HttpResponse::Ok().json(respData);
}

#[get("/{id}/poll")]
pub async fn get_poll(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
    path: web::Path<GetSinglePostPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<PollData> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    match PostService::get_poll(&pool, &path.id, &claim.user_name).await {
        Ok(Some(poll)) => {
            respData.message = "".to_string();
            respData.server_message = None;
            respData.data = Some(poll);
            return HttpResponse::Ok().json(respData);
        }
        Ok(None) => {
            respData.message = "Post has no poll".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::NotFound().json(respData);
        }
        Err(err) => {
            log::error!("error getting poll {}", err.to_string());
            respData.message = "Error getting poll".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };
}

#[post("/{id}/poll/vote")]
pub async fn vote_poll(
    pool: Data<PgPool>,
    req: Json<VotePollReq>,
    claim: Option<ReqData<Claims>>,
    path: web::Path<GetSinglePostPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<PollData> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    if req.option_id.trim().is_empty() {
        respData.message = "option_id is required".to_string();
        respData.server_message = None;
        respData.data = None;
        return HttpResponse::BadRequest().json(respData);
    }

    match PostService::vote_in_poll(
        &pool,
        path.id.clone(),
        req.option_id.clone(),
        claim.user_name.clone(),
    )
    .await
    {
        Ok(poll) => {
            respData.message = "Vote recorded".to_string();
            respData.server_message = None;
            respData.data = Some(poll);
            let pool_clone = pool.get_ref().clone();
            let actor = claim.user_name.clone();
            let post_id = path.id.clone();
            actix_web::rt::spawn(async move {
                match PostService::get_post_owner(&pool_clone, &post_id).await {
                    Ok(Some(owner)) if owner != actor => {
                        NotificationService::push(
                            &pool_clone,
                            &owner,
                            "poll_vote",
                            "New vote on your poll",
                            &format!("@{} voted in your poll", actor),
                            Some(&format!("/home/post/{}", post_id)),
                        )
                        .await;
                    }
                    _ => {}
                }
            });
            return HttpResponse::Ok().json(respData);
        }
        Err(err) => {
            log::error!("error voting in poll {}", err.to_string());
            respData.message = "Error recording vote".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
    message: String,
}

// impl fmt::Display for ApiError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.message)
//     }
// }

// impl ResponseError for ApiError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::build(self.status_code()).json(self)
//     }
//
//     fn status_code(&self) -> actix_web::http::StatusCode {
//         actix_web::http::StatusCode::BAD_REQUEST
//     }
// }
