use actix_web::{get, web, HttpResponse};
use actix_web::web::{Data, ReqData};
use sqlx::PgPool;
use crate::models::response::GenericResp;
use crate::req_models::requests::UpdateProfileReq;
use crate::services::profile_service::MiniProfile;
use crate::utils::auth::Claims;

#[get("/samp")]
pub async fn sample_controller(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            resp_data.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    resp_data
                )
        }
    };
    return HttpResponse::Ok().json(resp_data)
}