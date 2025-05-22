use actix_web::{get, web, HttpResponse};
use actix_web::web::{Data, Path, ReqData};
use log::error;
use serde_derive::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::ref_click::RefClick;
use crate::models::response::GenericResp;
use crate::req_models::requests::UpdateProfileReq;
use crate::services::profile_service::MiniProfile;
use crate::utils::auth::Claims;
use crate::services::ref_click_service::RefClickService;
#[derive(Deserialize)]
pub struct RefLinkPath{
    pub code: String,
}
#[get("/create_ref_link/{code}")]
pub async fn create_ref_link(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    path:Path<RefLinkPath>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut resp_data = GenericResp::<RefClick> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    
    // get reflink
    let click_id = Uuid::new_v4().to_string();
    let data = match RefClickService::create_ref_click(&pool, click_id,  path.code.to_owned()).await{
        Ok(data) => {data},
        Err(err)=>{
            error!("{}", err);
            resp_data.message = "Error creating ref link".to_string();
            return HttpResponse::InternalServerError().json(resp_data)
        }
    };

    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(data);
    return HttpResponse::Ok().json(resp_data)
}