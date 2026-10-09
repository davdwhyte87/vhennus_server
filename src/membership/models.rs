use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub mod status {
    pub const SUBMITTED: &str = "submitted";
    pub const UNDER_REVIEW: &str = "under_review";
    pub const APPROVED: &str = "approved";
    pub const REJECTED: &str = "rejected";

    pub fn is_pending(s: &str) -> bool {
        s == SUBMITTED || s == UNDER_REVIEW
    }
}

// ---------- DB rows ----------

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MembershipQuestion {
    pub id: String,
    pub question: String,
    pub is_required: bool,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MembershipQuestionOption {
    pub id: String,
    pub question_id: String,
    pub option_text: String,
    pub is_correct: bool,
    pub display_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MembershipApplication {
    pub id: String,
    pub user_name: String,
    pub status: String,
    pub score: i32,
    pub submitted_at: NaiveDateTime,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub decision_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MembershipApplicationAnswer {
    pub id: String,
    pub application_id: String,
    pub question_id: String,
    pub option_id: String,
    pub created_at: NaiveDateTime,
}

// Application row joined with the applicant profile contact info (for admin views).
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ApplicationWithContact {
    pub id: String,
    pub user_name: String,
    pub status: String,
    pub score: i32,
    pub submitted_at: NaiveDateTime,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub decision_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub country_of_origin: Option<String>,
    pub state_of_origin: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub current_country: Option<String>,
}

// One row per question with the applicant's selected option (if any).
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AnswerDetailRow {
    pub question_id: String,
    pub question: String,
    pub is_required: bool,
    pub option_id: Option<String>,
    pub option_text: Option<String>,
    pub is_correct: Option<bool>,
}

// ---------- Public DTOs (never expose is_correct) ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicOption {
    pub id: String,
    pub option_text: String,
    pub display_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicQuestion {
    pub id: String,
    pub question: String,
    pub is_required: bool,
    pub display_order: i32,
    pub options: Vec<PublicOption>,
}

// ---------- Admin DTOs ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminAnswerView {
    pub question_id: String,
    pub question: String,
    pub is_required: bool,
    pub option_id: Option<String>,
    pub option_text: Option<String>,
    pub is_correct: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminApplicationDetail {
    pub application: ApplicationWithContact,
    pub total_questions: i64,
    pub answers: Vec<AdminAnswerView>,
}

// ---------- Admin question management (is_correct visible to admins) ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminOptionView {
    pub id: String,
    pub option_text: String,
    pub is_correct: bool,
    pub display_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminQuestionView {
    pub id: String,
    pub question: String,
    pub is_required: bool,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
    pub options: Vec<AdminOptionView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewOptionReq {
    pub option_text: String,
    pub is_correct: Option<bool>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateQuestionReq {
    pub question: String,
    pub is_required: Option<bool>,
    pub display_order: Option<i32>,
    pub options: Vec<NewOptionReq>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateQuestionReq {
    pub question: Option<String>,
    pub is_required: Option<bool>,
    pub display_order: Option<i32>,
    // When present, REPLACES all options of the question.
    pub options: Option<Vec<NewOptionReq>>,
}

// ---------- Admin dashboard ----------

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AdminUserRow {
    pub user_name: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub membership: bool,
    pub phone_number: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminStatsResp {
    pub total_users: i64,
    pub total_members: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MembershipSettings {
    pub applications_paused: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSettingsReq {
    pub applications_paused: bool,
}

// ---------- Request bodies ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MembershipStatusResp {
    pub is_member: bool,
    pub applications_paused: bool,
    pub application: Option<MembershipApplication>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateApplicationReq {
    pub phone_number: Option<String>,
    pub country_of_origin: Option<String>,
    pub state_of_origin: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub current_country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerItem {
    pub question_id: String,
    pub option_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitAnswersReq {
    pub answers: Vec<AnswerItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionReq {
    pub reason: Option<String>,
}
