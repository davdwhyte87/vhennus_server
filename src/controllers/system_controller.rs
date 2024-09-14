use std::str::FromStr;

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{payment_method::{PaymentMethod, PaymentMethodData}, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}, system::System}, req_models::{create_payment_method_req::CreatePaymentMethodReq, create_sell_order_req::{CreateSellOrderReq, UpdateSellOrderReq}}, services::{mongo_service::MongoService, payment_method_service::PaymentMethodService, sell_order_service::SellOrderService, system_service::SystemService}, utils::auth::Claims};



#[get("/get_system_data")]
pub async fn get_system_data(

    database:Data<MongoService>
)->HttpResponse{
    let mut respData = GenericResp::<System>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    // get system data from db
    let data = match SystemService::get_system_data(&database.db).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting system data  {}", err.to_string());
            println!("Error getting system data {}", err);
            respData.message = "Error getting system data ".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    match data {
        Some(_)=>{},
        None=>{
            respData.message = "No system data".to_string();
            respData.server_message =None;
            respData.data = None;
        
            return HttpResponse::NotFound().json(respData)  
        }
    }
    respData.message = "Ok ".to_string();
    respData.server_message =None;
    respData.data = data;

    return HttpResponse::Ok().json(respData)


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