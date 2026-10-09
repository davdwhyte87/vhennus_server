use log::error;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};

use crate::membership::models::{
    status, AdminAnswerView, AdminApplicationDetail, AdminQuestionView, AdminStatsResp,
    AdminUserRow, AnswerItem, ApplicationWithContact, CreateQuestionReq, MembershipApplication,
    PublicOption, PublicQuestion, UpdateQuestionReq,
};
use crate::membership::repository::MembershipRepo;
use crate::profile::service::ProfileService;
use crate::shared::auth::Claims;
use crate::shared::error::AppError;
use crate::shared::send_email::EmailService;
use crate::user::service::UserService;

pub struct MembershipService;

impl MembershipService {
    fn ensure_admin(claim: &Claims) -> Result<(), AppError> {
        if claim.role != "ADMIN" {
            return Err(AppError::UnauthorizedError);
        }
        Ok(())
    }

    pub async fn get_public_questions(
        pool: &PgPool,
    ) -> Result<Vec<PublicQuestion>, AppError> {
        let questions = MembershipRepo::get_all_questions(pool).await?;
        let ids: Vec<String> = questions.iter().map(|q| q.id.clone()).collect();
        let options = MembershipRepo::get_options_for_questions(pool, &ids).await?;

        let mut by_question: HashMap<String, Vec<PublicOption>> = HashMap::new();
        for opt in options {
            by_question
                .entry(opt.question_id.clone())
                .or_default()
                .push(PublicOption {
                    id: opt.id,
                    option_text: opt.option_text,
                    display_order: opt.display_order,
                });
        }

        let mut result: Vec<PublicQuestion> = questions
            .into_iter()
            .map(|q| {
                let opts = by_question.remove(&q.id).unwrap_or_default();
                PublicQuestion {
                    id: q.id,
                    question: q.question,
                    is_required: q.is_required,
                    display_order: q.display_order,
                    options: opts,
                }
            })
            .collect();
        result.sort_by(|a, b| {
            a.display_order
                .cmp(&b.display_order)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(result)
    }

    pub async fn create_application(
        pool: &PgPool,
        claim: &Claims,
        body: crate::membership::models::CreateApplicationReq,
    ) -> Result<MembershipApplication, AppError> {
        let profile = ProfileService::get_profile(pool, claim.user_name.clone())
            .await
            .map_err(|err| {
                error!("error getting profile for membership: {}", err);
                AppError::NotFoundError(
                    "profiles".to_string(),
                    claim.user_name.clone(),
                )
            })?;
        if profile.membership {
            return Err(AppError::BadRequestError(
                "You are already a member".to_string(),
            ));
        }
        if let Some(pending) =
            MembershipRepo::get_pending_by_user(pool, claim.user_name.clone()).await?
        {
            error!(
                "duplicate membership application for {}",
                claim.user_name
            );
            let _ = pending;
            return Err(AppError::AlreadyExistsError);
        }

        let phone = body.phone_number.clone()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty());
        let country = body.country_of_origin.clone()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());
        let state = body.state_of_origin.clone()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let current_country = body.current_country.clone()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());
        if phone.is_some() || country.is_some() || state.is_some() || body.date_of_birth.is_some() || current_country.is_some() {
            ProfileService::update_contact_info(
                pool,
                claim.user_name.clone(),
                phone,
                country,
                state,
                body.date_of_birth,
                current_country,
            )
            .await
            .map_err(|err| {
                error!("error saving contact info: {}", err);
                AppError::DBUpdateError
            })?;
        }

        MembershipRepo::create_application(pool, claim.user_name.clone()).await
    }

    pub async fn get_my_application(
        pool: &PgPool,
        claim: &Claims,
    ) -> Result<Option<MembershipApplication>, AppError> {
        MembershipRepo::get_latest_by_user(pool, claim.user_name.clone()).await
    }

    /// Fresh membership status: profile.membership flag + latest application.
    /// Frontend should prefer this over the (long-lived) JWT claim.
    pub async fn get_status(
        pool: &PgPool,
        claim: &Claims,
    ) -> Result<crate::membership::models::MembershipStatusResp, AppError> {
        let profile = ProfileService::get_profile(pool, claim.user_name.clone())
            .await
            .map_err(|err| {
                error!("error getting profile for membership status: {}", err);
                AppError::NotFoundError(
                    "profiles".to_string(),
                    claim.user_name.clone(),
                )
            })?;
        let application =
            MembershipRepo::get_latest_by_user(pool, claim.user_name.clone()).await?;
        Ok(crate::membership::models::MembershipStatusResp {
            is_member: profile.membership,
            application,
        })
    }

    pub async fn submit_answers(
        pool: &PgPool,
        claim: &Claims,
        application_id: String,
        answers: Vec<AnswerItem>,
    ) -> Result<MembershipApplication, AppError> {
        let app = MembershipRepo::get_by_id(pool, application_id.clone()).await?;
        if app.user_name != claim.user_name {
            return Err(AppError::UnauthorizedError);
        }
        if !status::is_pending(&app.status) {
            return Err(AppError::BadRequestError(
                "This application can no longer be updated".to_string(),
            ));
        }
        if answers.is_empty() {
            return Err(AppError::BadRequestError(
                "No answers submitted".to_string(),
            ));
        }

        // All submitted question ids must exist; all required questions answered.
        let questions = MembershipRepo::get_all_questions(pool).await?;
        let required: HashSet<String> = questions
            .iter()
            .filter(|q| q.is_required)
            .map(|q| q.id.clone())
            .collect();
        let known: HashSet<String> = questions.iter().map(|q| q.id.clone()).collect();
        let answered: HashSet<String> =
            answers.iter().map(|a| a.question_id.clone()).collect();
        for qid in &answered {
            if !known.contains(qid) {
                return Err(AppError::BadRequestError(format!(
                    "Unknown question: {}",
                    qid
                )));
            }
        }
        for qid in &required {
            if !answered.contains(qid) {
                return Err(AppError::BadRequestError(format!(
                    "Required question not answered: {}",
                    qid
                )));
            }
        }

        // Every option must belong to its claimed question; score = #correct.
        let option_ids: Vec<String> =
            answers.iter().map(|a| a.option_id.clone()).collect();
        let options = MembershipRepo::get_options_by_ids(pool, &option_ids).await?;
        if options.len() != option_ids.len() {
            return Err(AppError::BadRequestError(
                "One or more selected options do not exist".to_string(),
            ));
        }
        let by_id: HashMap<String, (String, bool)> = options
            .into_iter()
            .map(|o| (o.id.clone(), (o.question_id.clone(), o.is_correct)))
            .collect();
        let mut score: i32 = 0;
        for a in &answers {
            match by_id.get(&a.option_id) {
                Some((owner_q, correct)) => {
                    if owner_q != &a.question_id {
                        return Err(AppError::BadRequestError(format!(
                            "Option does not belong to question {}",
                            a.question_id
                        )));
                    }
                    if *correct {
                        score += 1;
                    }
                }
                None => {
                    return Err(AppError::BadRequestError(
                        "One or more selected options do not exist".to_string(),
                    ));
                }
            }
        }

        let pairs: Vec<(String, String)> = answers
            .into_iter()
            .map(|a| (a.question_id, a.option_id))
            .collect();
        MembershipRepo::replace_answers(pool, application_id, &pairs, score).await
    }

    pub async fn list_applications(
        pool: &PgPool,
        claim: &Claims,
        status_filter: Option<String>,
    ) -> Result<Vec<ApplicationWithContact>, AppError> {
        Self::ensure_admin(claim)?;
        if let Some(ref s) = status_filter {
            if ![
                status::SUBMITTED,
                status::UNDER_REVIEW,
                status::APPROVED,
                status::REJECTED,
            ]
            .contains(&s.as_str())
            {
                return Err(AppError::BadRequestError(
                    "Invalid status filter".to_string(),
                ));
            }
        }
        MembershipRepo::list_applications(pool, status_filter).await
    }

    pub async fn get_application_detail(
        pool: &PgPool,
        claim: &Claims,
        application_id: String,
    ) -> Result<AdminApplicationDetail, AppError> {
        Self::ensure_admin(claim)?;
        let application =
            MembershipRepo::get_application_with_contact(pool, application_id.clone())
                .await?;
        let rows = MembershipRepo::get_answer_details(pool, application_id).await?;
        let total = rows.len() as i64;
        let answers = rows
            .into_iter()
            .map(|r| AdminAnswerView {
                question_id: r.question_id,
                question: r.question,
                is_required: r.is_required,
                option_id: r.option_id,
                option_text: r.option_text,
                is_correct: r.is_correct,
            })
            .collect();
        Ok(AdminApplicationDetail {
            application,
            total_questions: total,
            answers,
        })
    }

    pub async fn approve(
        pool: &PgPool,
        claim: &Claims,
        application_id: String,
        reason: Option<String>,
    ) -> Result<(), AppError> {
        Self::ensure_admin(claim)?;
        let app = MembershipRepo::get_by_id(pool, application_id.clone()).await?;
        if app.status == status::SUBMITTED {
            return Err(AppError::BadRequestError(
                "Take the application to under review first".to_string(),
            ));
        }
        if !status::is_pending(&app.status) {
            return Err(AppError::BadRequestError(
                "This application has already been decided".to_string(),
            ));
        }
        MembershipRepo::decide_application(
            pool,
            application_id,
            status::APPROVED.to_string(),
            claim.user_name.clone(),
            reason,
            true,
            app.user_name.clone(),
        )
        .await?;

        // Welcome email (best-effort: never fail the approval over mail).
        let email_service = EmailService::new();
        match UserService::get_by_username(pool, app.user_name.clone()).await {
            Ok(Some(user)) => {
                if let Some(email) = user.email {
                    if let Err(err) = email_service
                        .send_membership_approved_email(email, app.user_name.clone())
                        .await
                    {
                        error!("error sending membership approval email: {}", err);
                    }
                } else {
                    error!(
                        "no email on record for approved member {}",
                        app.user_name
                    );
                }
            }
            Ok(None) => {
                error!("approved member not found in users: {}", app.user_name);
            }
            Err(err) => {
                error!("error fetching approved member email: {}", err);
            }
        }
        Ok(())
    }

    pub async fn reject(
        pool: &PgPool,
        claim: &Claims,
        application_id: String,
        reason: Option<String>,
    ) -> Result<(), AppError> {
        Self::ensure_admin(claim)?;
        let app = MembershipRepo::get_by_id(pool, application_id.clone()).await?;
        if !status::is_pending(&app.status) {
            return Err(AppError::BadRequestError(
                "This application has already been decided".to_string(),
            ));
        }
        MembershipRepo::decide_application(
            pool,
            application_id,
            status::REJECTED.to_string(),
            claim.user_name.clone(),
            reason,
            false,
            app.user_name,
        )
        .await
    }

    pub async fn take_under_review(
        pool: &PgPool,
        claim: &Claims,
        application_id: String,
    ) -> Result<(), AppError> {
        Self::ensure_admin(claim)?;
        let app = MembershipRepo::get_by_id(pool, application_id.clone()).await?;
        if app.status != status::SUBMITTED {
            return Err(AppError::BadRequestError(
                "Only submitted applications can be taken to under review".to_string(),
            ));
        }
        MembershipRepo::mark_under_review(
            pool,
            application_id,
            claim.user_name.clone(),
        )
        .await
    }

    // ---------- Admin: questions ----------

    pub async fn list_questions_admin(
        pool: &PgPool,
        claim: &Claims,
    ) -> Result<Vec<AdminQuestionView>, AppError> {
        Self::ensure_admin(claim)?;
        MembershipRepo::get_admin_questions(pool).await
    }

    pub async fn create_question(
        pool: &PgPool,
        claim: &Claims,
        body: CreateQuestionReq,
    ) -> Result<AdminQuestionView, AppError> {
        Self::ensure_admin(claim)?;
        MembershipRepo::create_question_full(
            pool,
            body.question,
            body.is_required.unwrap_or(true),
            body.display_order.unwrap_or(0),
            &body.options,
        )
        .await
    }

    pub async fn update_question(
        pool: &PgPool,
        claim: &Claims,
        question_id: String,
        body: UpdateQuestionReq,
    ) -> Result<AdminQuestionView, AppError> {
        Self::ensure_admin(claim)?;
        MembershipRepo::update_question_full(
            pool,
            question_id,
            body.question,
            body.is_required,
            body.display_order,
            body.options.as_deref(),
        )
        .await
    }

    pub async fn delete_question(
        pool: &PgPool,
        claim: &Claims,
        question_id: String,
    ) -> Result<(), AppError> {
        Self::ensure_admin(claim)?;
        MembershipRepo::delete_question(pool, question_id).await
    }

    // ---------- Admin: dashboard ----------

    pub async fn get_stats(
        pool: &PgPool,
        claim: &Claims,
    ) -> Result<AdminStatsResp, AppError> {
        Self::ensure_admin(claim)?;
        let (total_users, total_members) =
            MembershipRepo::get_user_stats(pool).await?;
        Ok(AdminStatsResp {
            total_users,
            total_members,
        })
    }

    pub async fn list_users(
        pool: &PgPool,
        claim: &Claims,
    ) -> Result<Vec<AdminUserRow>, AppError> {
        Self::ensure_admin(claim)?;
        MembershipRepo::list_users_admin(pool).await
    }
}
