use std::collections::HashMap;

use actix_web::{delete, get, post, put, web, HttpResponse};
use actix_web::web::{Data, ReqData};
use log::error;
use serde::Serialize;
use sqlx::PgPool;

use crate::membership::models::{
    AdminApplicationDetail, AdminQuestionView, AdminStatsResp, AdminUserRow,
    ApplicationWithContact, CreateApplicationReq, CreateQuestionReq, DecisionReq,
    MembershipApplication, MembershipStatusResp, PublicQuestion, SubmitAnswersReq,
    UpdateQuestionReq,
};
use crate::membership::service::MembershipService;
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

fn claim_or_401<T: Serialize>(claim: Option<ReqData<Claims>>) -> Result<Claims, HttpResponse> {
    match claim {
        Some(c) => Ok(c.into_inner()),
        None => Err(unauthorized::<T>()),
    }
}

fn app_error_response<T: Serialize>(err: AppError, default_msg: &str) -> HttpResponse {
    let mut message = default_msg.to_string();
    let mut status = HttpResponse::InternalServerError();
    match &err {
        AppError::UnauthorizedError => {
            message = "You are not authorized".to_string();
            status = HttpResponse::Unauthorized();
        }
        AppError::NotFoundError(_, _) => {
            message = "Not found".to_string();
            status = HttpResponse::NotFound();
        }
        AppError::AlreadyExistsError => {
            message = "You already have a pending membership application".to_string();
            status = HttpResponse::BadRequest();
        }
        AppError::BadRequestError(data) => {
            message = data.clone();
            status = HttpResponse::BadRequest();
        }
        other => {
            error!("membership error: {}", other);
        }
    }
    status.json(GenericResp::<T> {
        message,
        server_message: None,
        data: None,
    })
}

// GET /membership/questions (never includes is_correct)
#[get("/questions")]
pub async fn get_questions(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    if claim_or_401::<Vec<PublicQuestion>>(claim).is_err() {
        return unauthorized::<Vec<PublicQuestion>>();
    }
    match MembershipService::get_public_questions(&pool).await {
        Ok(questions) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(questions),
        }),
        Err(err) => app_error_response::<Vec<PublicQuestion>>(
            err,
            "Error getting membership questions",
        ),
    }
}

// POST /membership/applications
#[post("/applications")]
pub async fn create_application(
    pool: Data<PgPool>,
    body: web::Json<CreateApplicationReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<MembershipApplication>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::create_application(&pool, &claims, body.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(app),
        }),
        Err(err) => {
            app_error_response::<MembershipApplication>(err, "Error creating application")
        }
    }
}

// GET /membership/my_application
#[get("/my_application")]
pub async fn my_application(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<MembershipApplication>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::get_my_application(&pool, &claims).await {
        Ok(app) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: app,
        }),
        Err(err) => app_error_response::<MembershipApplication>(
            err,
            "Error getting membership application",
        ),
    }
}

// GET /membership/status (fresh membership flag + latest application)
#[get("/status")]
pub async fn membership_status(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<MembershipStatusResp>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::get_status(&pool, &claims).await {
        Ok(status) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(status),
        }),
        Err(err) => app_error_response::<MembershipStatusResp>(
            err,
            "Error getting membership status",
        ),
    }
}

// POST /membership/applications/{id}/answers
// NOTE: accepts MULTIPLE answers at once ({ answers: [{question_id, option_id}, ...] })
#[post("/applications/{id}/answers")]
pub async fn submit_answers(
    pool: Data<PgPool>,
    path: web::Path<String>,
    body: web::Json<SubmitAnswersReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<MembershipApplication>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let application_id = path.into_inner();
    match MembershipService::submit_answers(&pool, &claims, application_id, body.answers.clone())
        .await
    {
        // Re-fetch answers without body move issues: SubmitAnswersReq.answers moved above via clone
        Ok(app) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(app),
        }),
        Err(err) => {
            app_error_response::<MembershipApplication>(err, "Error submitting answers")
        }
    }
}

// GET /admin/membership/applications?status=
#[get("/applications")]
pub async fn list_applications(
    pool: Data<PgPool>,
    query: web::Query<HashMap<String, String>>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<Vec<ApplicationWithContact>>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let status = query.get("status").cloned();
    match MembershipService::list_applications(&pool, &claims, status).await {
        Ok(apps) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(apps),
        }),
        Err(err) => app_error_response::<Vec<ApplicationWithContact>>(
            err,
            "Error listing membership applications",
        ),
    }
}

// GET /admin/membership/applications/{id}
#[get("/applications/{id}")]
pub async fn get_application(
    pool: Data<PgPool>,
    path: web::Path<String>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<AdminApplicationDetail>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::get_application_detail(&pool, &claims, path.into_inner()).await
    {
        Ok(detail) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(detail),
        }),
        Err(err) => app_error_response::<AdminApplicationDetail>(
            err,
            "Error getting membership application",
        ),
    }
}

// POST /admin/membership/applications/{id}/review (submitted -> under_review)
#[post("/applications/{id}/review")]
pub async fn review_application(
    pool: Data<PgPool>,
    path: web::Path<String>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<String>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::take_under_review(&pool, &claims, path.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(GenericResp::<String> {
            message: "Application under review".to_string(),
            server_message: None,
            data: None,
        }),
        Err(err) => app_error_response::<String>(err, "Error reviewing application"),
    }
}

// POST /admin/membership/applications/{id}/approve (under_review -> approved)
#[post("/applications/{id}/approve")]
pub async fn approve_application(
    pool: Data<PgPool>,
    path: web::Path<String>,
    body: Option<web::Json<DecisionReq>>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<String>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let reason = body.and_then(|b| b.reason.clone());
    match MembershipService::approve(&pool, &claims, path.into_inner(), reason).await {
        Ok(_) => HttpResponse::Ok().json(GenericResp::<String> {
            message: "Membership approved".to_string(),
            server_message: None,
            data: None,
        }),
        Err(err) => app_error_response::<String>(err, "Error approving application"),
    }
}

// POST /admin/membership/applications/{id}/reject
#[post("/applications/{id}/reject")]
pub async fn reject_application(
    pool: Data<PgPool>,
    path: web::Path<String>,
    body: Option<web::Json<DecisionReq>>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<String>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let reason = body.and_then(|b| b.reason.clone());
    match MembershipService::reject(&pool, &claims, path.into_inner(), reason).await {
        Ok(_) => HttpResponse::Ok().json(GenericResp::<String> {
            message: "Membership rejected".to_string(),
            server_message: None,
            data: None,
        }),
        Err(err) => app_error_response::<String>(err, "Error rejecting application"),
    }
}

// GET /admin/membership/questions (includes is_correct, admin only)
#[get("/questions")]
pub async fn admin_list_questions(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<Vec<AdminQuestionView>>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::list_questions_admin(&pool, &claims).await {
        Ok(questions) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(questions),
        }),
        Err(err) => app_error_response::<Vec<AdminQuestionView>>(
            err,
            "Error listing membership questions",
        ),
    }
}

// POST /admin/membership/questions
#[post("/questions")]
pub async fn admin_create_question(
    pool: Data<PgPool>,
    body: web::Json<CreateQuestionReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<AdminQuestionView>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::create_question(&pool, &claims, body.into_inner()).await {
        Ok(question) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(question),
        }),
        Err(err) => app_error_response::<AdminQuestionView>(
            err,
            "Error creating membership question",
        ),
    }
}

// PUT /admin/membership/questions/{id}
#[put("/questions/{id}")]
pub async fn admin_update_question(
    pool: Data<PgPool>,
    path: web::Path<String>,
    body: web::Json<UpdateQuestionReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<AdminQuestionView>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::update_question(
        &pool,
        &claims,
        path.into_inner(),
        body.into_inner(),
    )
    .await
    {
        Ok(question) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(question),
        }),
        Err(err) => app_error_response::<AdminQuestionView>(
            err,
            "Error updating membership question",
        ),
    }
}

// DELETE /admin/membership/questions/{id}
#[delete("/questions/{id}")]
pub async fn admin_delete_question(
    pool: Data<PgPool>,
    path: web::Path<String>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<String>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::delete_question(&pool, &claims, path.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(GenericResp::<String> {
            message: "Question deleted".to_string(),
            server_message: None,
            data: None,
        }),
        Err(err) => app_error_response::<String>(err, "Error deleting question"),
    }
}

// GET /admin/membership/stats
#[get("/stats")]
pub async fn admin_stats(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<AdminStatsResp>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::get_stats(&pool, &claims).await {
        Ok(stats) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(stats),
        }),
        Err(err) => {
            app_error_response::<AdminStatsResp>(err, "Error getting stats")
        }
    }
}

// GET /admin/membership/users
#[get("/users")]
pub async fn admin_list_users(
    pool: Data<PgPool>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let claims = match claim_or_401::<Vec<AdminUserRow>>(claim) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match MembershipService::list_users(&pool, &claims).await {
        Ok(users) => HttpResponse::Ok().json(GenericResp {
            message: "Ok".to_string(),
            server_message: None,
            data: Some(users),
        }),
        Err(err) => {
            app_error_response::<Vec<AdminUserRow>>(err, "Error listing users")
        }
    }
}
