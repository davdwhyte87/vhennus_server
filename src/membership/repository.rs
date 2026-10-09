use log::error;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::membership::models::{
    AdminQuestionView, AdminOptionView, AdminUserRow, AnswerDetailRow, ApplicationWithContact,
    MembershipApplication, MembershipQuestion, MembershipQuestionOption, MembershipSettings,
    NewOptionReq,
};
use crate::shared::error::AppError;
use crate::shared::general::get_time_naive;

pub struct MembershipRepo;

impl MembershipRepo {
    pub async fn get_all_questions(
        pool: &PgPool,
    ) -> Result<Vec<MembershipQuestion>, AppError> {
        let questions = sqlx::query_as!(
            MembershipQuestion,
            "SELECT * FROM membership_questions ORDER BY display_order ASC, id ASC"
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("error fetching membership questions: {}", err);
            AppError::FetchDataError
        })?;
        Ok(questions)
    }

    pub async fn get_options_for_questions(
        pool: &PgPool,
        question_ids: &[String],
    ) -> Result<Vec<MembershipQuestionOption>, AppError> {
        if question_ids.is_empty() {
            return Ok(vec![]);
        }
        let options = sqlx::query_as!(
            MembershipQuestionOption,
            "SELECT * FROM membership_question_options
             WHERE question_id = ANY($1)
             ORDER BY display_order ASC, id ASC",
            question_ids
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("error fetching membership options: {}", err);
            AppError::FetchDataError
        })?;
        Ok(options)
    }

    pub async fn get_options_by_ids(
        pool: &PgPool,
        option_ids: &[String],
    ) -> Result<Vec<MembershipQuestionOption>, AppError> {
        if option_ids.is_empty() {
            return Ok(vec![]);
        }
        let options = sqlx::query_as!(
            MembershipQuestionOption,
            "SELECT * FROM membership_question_options WHERE id = ANY($1)",
            option_ids
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("error fetching membership options by ids: {}", err);
            AppError::FetchDataError
        })?;
        Ok(options)
    }

    pub async fn create_application(
        pool: &PgPool,
        user_name: String,
    ) -> Result<MembershipApplication, AppError> {
        let now = get_time_naive();
        let app = MembershipApplication {
            id: Uuid::new_v4().to_string(),
            user_name: user_name.clone(),
            status: crate::membership::models::status::SUBMITTED.to_string(),
            score: 0,
            submitted_at: now,
            reviewed_by: None,
            reviewed_at: None,
            decision_reason: None,
            created_at: now,
            updated_at: now,
        };
        let res = sqlx::query!(
            "INSERT INTO membership_applications
             (id, user_name, status, score, submitted_at, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
            app.id,
            app.user_name,
            app.status,
            app.score,
            app.submitted_at,
            app.created_at,
            app.updated_at,
        )
        .execute(pool)
        .await;
        match res {
            Ok(_) => Ok(app),
            Err(err) => {
                error!("error creating membership application: {}", err);
                if let Some(db_err) = err.as_database_error() {
                    if db_err.code().as_deref() == Some("23505") {
                        return Err(AppError::AlreadyExistsError);
                    }
                }
                Err(AppError::DBInsertError)
            }
        }
    }

    pub async fn get_pending_by_user(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Option<MembershipApplication>, AppError> {
        let app = sqlx::query_as!(
            MembershipApplication,
            "SELECT * FROM membership_applications
             WHERE user_name = $1 AND status IN ('submitted','under_review')
             ORDER BY created_at DESC LIMIT 1",
            user_name
        )
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            error!("error fetching pending membership application: {}", err);
            AppError::FetchDataError
        })?;
        Ok(app)
    }

    pub async fn get_latest_by_user(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Option<MembershipApplication>, AppError> {
        let app = sqlx::query_as!(
            MembershipApplication,
            "SELECT * FROM membership_applications
             WHERE user_name = $1
             ORDER BY created_at DESC LIMIT 1",
            user_name
        )
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            error!("error fetching latest membership application: {}", err);
            AppError::FetchDataError
        })?;
        Ok(app)
    }

    pub async fn get_by_id(
        pool: &PgPool,
        id: String,
    ) -> Result<MembershipApplication, AppError> {
        let app = sqlx::query_as!(
            MembershipApplication,
            "SELECT * FROM membership_applications WHERE id = $1",
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => {
                AppError::NotFoundError("membership_applications".to_string(), id)
            }
            other => {
                error!("error fetching membership application: {}", other);
                AppError::FetchDataError
            }
        })?;
        Ok(app)
    }

    pub async fn replace_answers(
        pool: &PgPool,
        application_id: String,
        answers: &[(String, String)],
        score: i32,
    ) -> Result<MembershipApplication, AppError> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(|err| {
            error!("db-transaction error: {}", err);
            AppError::CreateTransactionError
        })?;

        sqlx::query!(
            "DELETE FROM membership_application_answers WHERE application_id = $1",
            application_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!("error clearing old membership answers: {}", err);
            AppError::DBDeleteError
        })?;

        for (question_id, option_id) in answers {
            sqlx::query!(
                "INSERT INTO membership_application_answers (id, application_id, question_id, option_id)
                 VALUES ($1,$2,$3,$4)",
                Uuid::new_v4().to_string(),
                application_id,
                question_id,
                option_id,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!("error inserting membership answer: {}", err);
                AppError::DBInsertError
            })?;
        }

        let now = get_time_naive();
        let app = sqlx::query_as!(
            MembershipApplication,
            "UPDATE membership_applications
             SET score = $2, submitted_at = $3, updated_at = $3
             WHERE id = $1
             RETURNING id, user_name, status, score, submitted_at,
                       reviewed_by, reviewed_at, decision_reason, created_at, updated_at",
            application_id,
            score,
            now,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| {
            error!("error updating membership score: {}", err);
            AppError::DBUpdateError
        })?;

        tx.commit().await.map_err(|err| {
            error!("error committing membership answers: {}", err);
            AppError::CreateTransactionError
        })?;
        Ok(app)
    }

    pub async fn list_applications(
        pool: &PgPool,
        status_filter: Option<String>,
    ) -> Result<Vec<ApplicationWithContact>, AppError> {
        let apps = if let Some(status) = status_filter {
            sqlx::query_as!(
                ApplicationWithContact,
                "SELECT a.id, a.user_name, a.status, a.score, a.submitted_at,
                        a.reviewed_by, a.reviewed_at, a.decision_reason,
                        a.created_at, a.updated_at, u.email, p.phone_number,
                        p.country_of_origin, p.state_of_origin, p.date_of_birth,
                        p.current_country
                 FROM membership_applications a
                 LEFT JOIN profiles p ON p.user_name = a.user_name
                 LEFT JOIN users u ON u.user_name = a.user_name
                 WHERE a.status = $1
                 ORDER BY a.submitted_at DESC",
                status
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as!(
                ApplicationWithContact,
                "SELECT a.id, a.user_name, a.status, a.score, a.submitted_at,
                        a.reviewed_by, a.reviewed_at, a.decision_reason,
                        a.created_at, a.updated_at, u.email, p.phone_number,
                        p.country_of_origin, p.state_of_origin, p.date_of_birth,
                        p.current_country
                 FROM membership_applications a
                 LEFT JOIN profiles p ON p.user_name = a.user_name
                 LEFT JOIN users u ON u.user_name = a.user_name
                 ORDER BY a.submitted_at DESC"
            )
            .fetch_all(pool)
            .await
        };
        let apps = apps.map_err(|err| {
            error!("error listing membership applications: {}", err);
            AppError::FetchDataError
        })?;
        Ok(apps)
    }

    pub async fn get_application_with_contact(
        pool: &PgPool,
        id: String,
    ) -> Result<ApplicationWithContact, AppError> {
        let app = sqlx::query_as!(
            ApplicationWithContact,
            "SELECT a.id, a.user_name, a.status, a.score, a.submitted_at,
                    a.reviewed_by, a.reviewed_at, a.decision_reason,
                    a.created_at, a.updated_at, u.email, p.phone_number,
                    p.country_of_origin, p.state_of_origin, p.date_of_birth,
                    p.current_country
             FROM membership_applications a
             LEFT JOIN profiles p ON p.user_name = a.user_name
             LEFT JOIN users u ON u.user_name = a.user_name
             WHERE a.id = $1",
            id
        )
        .fetch_one(pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => {
                AppError::NotFoundError("membership_applications".to_string(), id)
            }
            other => {
                error!("error fetching membership application detail: {}", other);
                AppError::FetchDataError
            }
        })?;
        Ok(app)
    }

    pub async fn get_answer_details(
        pool: &PgPool,
        application_id: String,
    ) -> Result<Vec<AnswerDetailRow>, AppError> {
        let rows = sqlx::query_as!(
            AnswerDetailRow,
            "SELECT q.id AS question_id, q.question, q.is_required,
                    ans.option_id, o.option_text, o.is_correct
             FROM membership_questions q
             LEFT JOIN membership_application_answers ans
               ON ans.question_id = q.id AND ans.application_id = $1
             LEFT JOIN membership_question_options o ON o.id = ans.option_id
             ORDER BY q.display_order ASC, q.id ASC",
            application_id
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("error fetching membership answer details: {}", err);
            AppError::FetchDataError
        })?;
        Ok(rows)
    }

    pub async fn decide_application(
        pool: &PgPool,
        application_id: String,
        new_status: String,
        reviewed_by: String,
        reason: Option<String>,
        set_member: bool,
        applicant_user_name: String,
    ) -> Result<(), AppError> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(|err| {
            error!("db-transaction error: {}", err);
            AppError::CreateTransactionError
        })?;

        let now = get_time_naive();
        let res = sqlx::query!(
            "UPDATE membership_applications
             SET status = $2, reviewed_by = $3, reviewed_at = $4,
                 decision_reason = $5, updated_at = $4
             WHERE id = $1",
            application_id,
            new_status,
            reviewed_by,
            now,
            reason,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!("error updating membership application status: {}", err);
            AppError::DBUpdateError
        })?;
        if res.rows_affected() == 0 {
            let _ = tx.rollback().await;
            return Err(AppError::NotFoundError(
                "membership_applications".to_string(),
                application_id,
            ));
        }

        sqlx::query!(
            "UPDATE profiles SET membership = $2, updated_at = NOW() WHERE user_name = $1",
            applicant_user_name,
            set_member,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!("error updating profile membership flag: {}", err);
            AppError::DBUpdateError
        })?;

        tx.commit().await.map_err(|err| {
            error!("error committing membership decision: {}", err);
            AppError::CreateTransactionError
        })?;
        Ok(())
    }

    pub async fn mark_under_review(
        pool: &PgPool,
        application_id: String,
        reviewed_by: String,
    ) -> Result<(), AppError> {
        let now = get_time_naive();
        let res = sqlx::query!(
            "UPDATE membership_applications
             SET status = 'under_review', reviewed_by = $2, reviewed_at = $3, updated_at = $3
             WHERE id = $1 AND status = 'submitted'",
            application_id,
            reviewed_by,
            now,
        )
        .execute(pool)
        .await
        .map_err(|err| {
            error!("error marking membership application under review: {}", err);
            AppError::DBUpdateError
        })?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFoundError(
                "membership_applications".to_string(),
                application_id,
            ));
        }
        Ok(())
    }

    // ---------- Admin: questions (with is_correct) ----------

    pub async fn get_admin_questions(
        pool: &PgPool,
    ) -> Result<Vec<AdminQuestionView>, AppError> {
        let questions = Self::get_all_questions(pool).await?;
        let ids: Vec<String> = questions.iter().map(|q| q.id.clone()).collect();
        let options = Self::get_options_for_questions(pool, &ids).await?;

        let mut by_question: std::collections::HashMap<String, Vec<AdminOptionView>> =
            std::collections::HashMap::new();
        for opt in options {
            by_question
                .entry(opt.question_id.clone())
                .or_default()
                .push(AdminOptionView {
                    id: opt.id,
                    option_text: opt.option_text,
                    is_correct: opt.is_correct,
                    display_order: opt.display_order,
                });
        }

        let mut result: Vec<AdminQuestionView> = questions
            .into_iter()
            .map(|q| {
                let mut opts = by_question.remove(&q.id).unwrap_or_default();
                opts.sort_by(|a, b| {
                    a.display_order.cmp(&b.display_order).then_with(|| a.id.cmp(&b.id))
                });
                AdminQuestionView {
                    id: q.id,
                    question: q.question,
                    is_required: q.is_required,
                    display_order: q.display_order,
                    created_at: q.created_at,
                    options: opts,
                }
            })
            .collect();
        // FIFO: first created shows first, last created shows last.
        result.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(result)
    }

    fn validated_options(
        options: &[NewOptionReq],
    ) -> Result<Vec<(String, bool, i32)>, AppError> {
        if options.is_empty() {
            return Err(AppError::BadRequestError(
                "A question must have at least one option".to_string(),
            ));
        }
        let mut out = Vec::with_capacity(options.len());
        for (i, o) in options.iter().enumerate() {
            if o.option_text.trim().is_empty() {
                return Err(AppError::BadRequestError(
                    "Option text cannot be empty".to_string(),
                ));
            }
            out.push((
                o.option_text.trim().to_string(),
                o.is_correct.unwrap_or(false),
                o.display_order.unwrap_or(i as i32),
            ));
        }
        Ok(out)
    }

    pub async fn create_question_full(
        pool: &PgPool,
        question: String,
        is_required: bool,
        _display_order: i32,
        options: &[NewOptionReq],
    ) -> Result<AdminQuestionView, AppError> {
        if question.trim().is_empty() {
            return Err(AppError::BadRequestError(
                "Question text cannot be empty".to_string(),
            ));
        }
        let opts = Self::validated_options(options)?;
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(|err| {
            error!("db-transaction error: {}", err);
            AppError::CreateTransactionError
        })?;

        // Display order is automatic: new questions go last.
        let display_order: i32 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(display_order), 0) + 1 FROM membership_questions"
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| {
            error!("error computing display order: {}", err);
            AppError::FetchDataError
        })?
        .unwrap_or(1);

        let now = get_time_naive();
        let qid = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO membership_questions (id, question, is_required, display_order, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6)",
            qid,
            question.trim(),
            is_required,
            display_order,
            now,
            now,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!("error creating membership question: {}", err);
            AppError::DBInsertError
        })?;

        let mut views = Vec::with_capacity(opts.len());
        for (text, correct, order) in &opts {
            let oid = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO membership_question_options (id, question_id, option_text, is_correct, display_order)
                 VALUES ($1,$2,$3,$4,$5)",
                oid,
                qid,
                text,
                *correct,
                *order,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!("error creating membership option: {}", err);
                AppError::DBInsertError
            })?;
            views.push(AdminOptionView {
                id: oid,
                option_text: text.clone(),
                is_correct: *correct,
                display_order: *order,
            });
        }
        views.sort_by(|a, b| {
            a.display_order.cmp(&b.display_order).then_with(|| a.id.cmp(&b.id))
        });

        tx.commit().await.map_err(|err| {
            error!("error committing membership question: {}", err);
            AppError::CreateTransactionError
        })?;
        Ok(AdminQuestionView {
            id: qid,
            question: question.trim().to_string(),
            is_required,
            display_order,
            created_at: now,
            options: views,
        })
    }

    pub async fn update_question_full(
        pool: &PgPool,
        question_id: String,
        question: Option<String>,
        is_required: Option<bool>,
        display_order: Option<i32>,
        options: Option<&[NewOptionReq]>,
    ) -> Result<AdminQuestionView, AppError> {
        if let Some(ref q) = question {
            if q.trim().is_empty() {
                return Err(AppError::BadRequestError(
                    "Question text cannot be empty".to_string(),
                ));
            }
        }
        let replacement = match options {
            Some(list) => Some(Self::validated_options(list)?),
            None => None,
        };

        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(|err| {
            error!("db-transaction error: {}", err);
            AppError::CreateTransactionError
        })?;

        let now = get_time_naive();
        if question.is_some() || is_required.is_some() || display_order.is_some() {
            let res = sqlx::query!(
                "UPDATE membership_questions
                 SET question = COALESCE($2, question),
                     is_required = COALESCE($3, is_required),
                     display_order = COALESCE($4, display_order),
                     updated_at = $5
                 WHERE id = $1",
                question_id,
                question.as_deref(),
                is_required,
                display_order,
                now,
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!("error updating membership question: {}", err);
                AppError::DBUpdateError
            })?;
            if res.rows_affected() == 0 {
                let _ = tx.rollback().await;
                return Err(AppError::NotFoundError(
                    "membership_questions".to_string(),
                    question_id,
                ));
            }
        } else {
            let exists: bool = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM membership_questions WHERE id = $1)",
                question_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| {
                error!("error checking membership question: {}", err);
                AppError::FetchDataError
            })?
            .unwrap_or(false);
            if !exists {
                let _ = tx.rollback().await;
                return Err(AppError::NotFoundError(
                    "membership_questions".to_string(),
                    question_id,
                ));
            }
        }

        if let Some(opts) = replacement {
            sqlx::query!(
                "DELETE FROM membership_question_options WHERE question_id = $1",
                question_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!("error clearing membership options: {}", err);
                AppError::DBDeleteError
            })?;
            for (text, correct, order) in &opts {
                sqlx::query!(
                    "INSERT INTO membership_question_options (id, question_id, option_text, is_correct, display_order)
                     VALUES ($1,$2,$3,$4,$5)",
                    Uuid::new_v4().to_string(),
                    question_id,
                    text,
                    *correct,
                    *order,
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    error!("error creating membership option: {}", err);
                    AppError::DBInsertError
                })?;
            }
        }

        tx.commit().await.map_err(|err| {
            error!("error committing membership question update: {}", err);
            AppError::CreateTransactionError
        })?;

        // Re-read the full admin view.
        let all = Self::get_admin_questions(pool).await?;
        all.into_iter()
            .find(|q| q.id == question_id)
            .ok_or(AppError::NotFoundError(
                "membership_questions".to_string(),
                question_id,
            ))
    }

    pub async fn delete_question(pool: &PgPool, question_id: String) -> Result<(), AppError> {
        let res = sqlx::query!(
            "DELETE FROM membership_questions WHERE id = $1",
            question_id
        )
        .execute(pool)
        .await
        .map_err(|err| {
            error!("error deleting membership question: {}", err);
            AppError::DBDeleteError
        })?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFoundError(
                "membership_questions".to_string(),
                question_id,
            ));
        }
        Ok(())
    }

    // ---------- Admin: dashboard ----------

    pub async fn get_user_stats(pool: &PgPool) -> Result<(i64, i64), AppError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) AS total, COUNT(*) FILTER (WHERE membership = TRUE) AS members FROM profiles"
        )
        .fetch_one(pool)
        .await
        .map_err(|err| {
            error!("error fetching user stats: {}", err);
            AppError::FetchDataError
        })?;
        Ok((row.total.unwrap_or(0), row.members.unwrap_or(0)))
    }

    pub async fn list_users_admin(pool: &PgPool) -> Result<Vec<AdminUserRow>, AppError> {
        let rows = sqlx::query_as!(
            AdminUserRow,
            "SELECT p.user_name, p.name, u.email, p.membership, p.phone_number, p.created_at
             FROM profiles p
             LEFT JOIN users u ON u.user_name = p.user_name
             ORDER BY p.created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("error listing users: {}", err);
            AppError::FetchDataError
        })?;
        Ok(rows)
    }

    // ---------- Membership application pause switch ----------

    pub async fn get_settings(pool: &PgPool) -> Result<MembershipSettings, AppError> {
        let settings = sqlx::query_as!(
            MembershipSettings,
            "SELECT applications_paused FROM membership_settings WHERE id = 1"
        )
        .fetch_optional(pool)
        .await
        .map_err(|err| {
            error!("error fetching membership settings: {}", err);
            AppError::FetchDataError
        })?
        .unwrap_or(MembershipSettings {
            applications_paused: false,
        });
        Ok(settings)
    }

    pub async fn are_applications_paused(pool: &PgPool) -> Result<bool, AppError> {
        Ok(Self::get_settings(pool).await?.applications_paused)
    }

    pub async fn set_applications_paused(
        pool: &PgPool,
        paused: bool,
    ) -> Result<MembershipSettings, AppError> {
        let settings = sqlx::query_as!(
            MembershipSettings,
            "INSERT INTO membership_settings (id, applications_paused, updated_at)
             VALUES (1, $1, NOW())
             ON CONFLICT (id) DO UPDATE SET applications_paused = $1, updated_at = NOW()
             RETURNING applications_paused",
            paused
        )
        .fetch_one(pool)
        .await
        .map_err(|err| {
            error!("error updating membership settings: {}", err);
            AppError::DBUpdateError
        })?;
        Ok(settings)
    }
}
