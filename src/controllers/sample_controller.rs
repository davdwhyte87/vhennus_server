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
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData);
        }
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };
    return HttpResponse::Ok().json({})
}