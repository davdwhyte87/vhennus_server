use actix_web::{get, post, web, HttpResponse};
use actix_web::web::{Data, ReqData};
use log::error;
use sqlx::PgPool;
use uuid::Uuid;
use crate::controllers::user_controller::resend_code;

use crate::models::response::GenericResp;
use crate::req_models::requests::{CreateGroupReq, UpdateProfileReq};

use crate::services::profile_service::MiniProfile;
use crate::utils::auth::Claims;
use crate::utils::general::get_time_naive;

