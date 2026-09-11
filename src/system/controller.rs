use std::str::FromStr;

use actix_web::{
    get, post,
    web::{self, Data, ReqData},
    HttpResponse,
};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::orders::models::{
    CreatePaymentMethodReq, CreateSellOrderReq, Currency, PaymentMethod, PaymentMethodData,
    SellOrder, UpdateSellOrderReq,
};
use crate::orders::service::{PaymentMethodService, SellOrderService};
use crate::shared::auth::Claims;
use crate::shared::db::MongoService;
use crate::shared::response::{GenericResp, Response};
use crate::system::models::System;
use crate::system::service::SystemService;

#[get("/get_system_data")]
pub async fn get_system_data(pool: Data<PgPool>) -> HttpResponse {
    let mut respData = GenericResp::<System> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    // get system data from db
    let data = match SystemService::get_system_data(&pool).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting system data  {}", err.to_string());
            println!("Error getting system data {}", err);
            respData.message = "Error getting system data ".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
    };

    match data {
        Some(_) => {}
        None => {
            respData.message = "No system data".to_string();
            respData.server_message = None;
            respData.data = None;

            return HttpResponse::NotFound().json(respData);
        }
    }
    respData.message = "Ok ".to_string();
    respData.server_message = None;
    respData.data = data;

    return HttpResponse::Ok().json(respData);
}

// #[post("/mmdjkks")]
// pub async fn sample(

//     database:Data<MongoService>,
//     req:Json<CreatePaymentMethodReq>,
//     claim:Option<ReqData<Claims>>
// )->HttpResponse{
//     let mut respData = GenericResp::<PaymentMethodData>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: None
//     };


//     return HttpResponse::Ok().json(respData)


// }