use std::str::FromStr;

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{payment_method::{PaymentMethod, PaymentMethodData}, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}}, req_models::{create_payment_method_req::CreatePaymentMethodReq, create_sell_order_req::{CreateSellOrderReq, UpdateSellOrderReq}}, services::{mongo_service::MongoService, payment_method_service::PaymentMethodService, sell_order_service::SellOrderService}, utils::auth::Claims};



#[post("/create")]
pub async fn create_payment_method(

    database:Data<MongoService>,
    req:Json<CreatePaymentMethodReq>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<PaymentMethodData>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    };
    let payment_method= PaymentMethodData{
        id: Uuid::new_v4().to_string(), 
        user_name: claim.user_name.to_owned(),
        payment_method: req.payment_method.to_owned(),
        account_name : req.account_name.to_owned(),
        account_number: req.account_number.to_owned(),
        bank_name: req.bank_name.to_owned(),
        other: req.other.to_owned(),
        paypal_email: req.paypal_email.to_owned(),
        venmo_username: req.venmo_username.to_owned(),
        skrill_email:req.skrill_email.to_owned(),
        name: req.name.to_owned()
    };

    match PaymentMethodService::create_payment_method(&database.db, &payment_method).await{
        Ok(_)=>{},
        Err(err)=>{
            log::error!(" error creating payment method {}", err.to_string());
            respData.message = "Error creating payment method".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::BadRequest().json(
             respData
            )  
        }
    }

    respData.message = "Ok".to_string();
    respData.data = Some(payment_method);
    respData.server_message = None;

    return HttpResponse::Ok().json(
     respData
    )  
}

#[derive(Deserialize)]
struct GetSingleSellOrderPath {
    id: String,
}

#[get("/delete/{id}")]
pub async fn delete_payment_method(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetSingleSellOrderPath>
)->HttpResponse
{

    let mut respData = GenericResp::<Vec<SellOrder>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
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
 

    match PaymentMethodService::delete_user_payment_method(&database.db, info.id.to_owned()).await{
        Ok(_)=>{},
        Err(err)=>{
            log::error!("error deleting payment method  {}", err.to_string());
            respData.message = "Error deleting payment method".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(
             respData
            )  
        }
    }
    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;

    return HttpResponse::Ok().json(
        respData
    )  
}


#[get("/my_payment_methods")]
pub async fn get_my_payment_methods(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
   
)->HttpResponse
{
    let mut respData = GenericResp::<Vec<PaymentMethodData>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Not authorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized()
                .json(respData)
        }
    };

    let methods = match PaymentMethodService::get_all_user_payment_method_data(&database.db, claim.user_name.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error  gettting user payment mthods {}", err.to_string());
            respData.message = "Error getting  payment method".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::BadRequest().json(
            respData
            )  
        } 
    };

    respData.message = "Ok".to_string();
    respData.data =Some(methods);
    respData.server_message = None;
    return HttpResponse::Ok().json(
      respData
    ) 
}