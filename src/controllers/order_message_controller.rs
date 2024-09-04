

use std::str::FromStr;

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::{num_bigint::BigInt, BigDecimal};
use lettre::transport::smtp::response;
use mongodb::bson::doc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{message::OrderMessage, payment_method::PaymentMethod, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}}, req_models::{create_sell_order_req::{CreateSellOrderReq, UpdateSellOrderReq}, requests::CreateOrderMessageReq}, services::{message_service::OrderMessageService, mongo_service::MongoService, sell_order_service::SellOrderService}, utils::auth::Claims};




#[post("/post")]
pub async fn create_order_message(
    database:Data<MongoService>,
    new_message:Json<CreateOrderMessageReq>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{


    let mut respData = GenericResp::<OrderMessage>{
            message:"".to_string(),
            server_message: Some("".to_string()),
            data: Some(OrderMessage::default())
        };
    println!("new req");

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

    let order_message = OrderMessage{
        id: Uuid::new_v4().to_string(),
        text: new_message.text.to_owned(),
        image: new_message.image.to_owned(),
        created_at: chrono::offset::Utc::now().to_string(),
        sender_user_name: claim.user_name.to_owned(),
        receiver_user_name: new_message.receiver_user_name.to_owned(), 
        buy_order_id: new_message.buy_order_id.to_owned()
    };

    let response = match OrderMessageService::create_message(&database.db, &order_message).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting order messages  {}", err.to_string());
            respData.data = None;
            respData.message = "Error creating message".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(
                respData
            );
        }
    };

    respData.data = Some(order_message);
    respData.message = "Created".to_string();
    respData.server_message = None;
    return HttpResponse::Ok().json(
        respData
    )
}


#[derive(Deserialize)]
pub struct  IDPath{
    pub id:String
}

#[get("/get_all/{id}")]
pub async fn get_order_message(
    database:Data<MongoService>,
    info: web::Path<IDPath>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{


    let mut respData = GenericResp::<Vec<OrderMessage>>{
            message:"".to_string(),
            server_message: Some("".to_string()),
            data: None
        };
    println!("new req");

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


    let filter =  doc! {"buy_order_id": info.id.to_owned()};
    let response = match OrderMessageService::get_message(&database.db, filter).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting messages  {}", err.to_string());
            respData.data = None;
            respData.message = "Error creating message".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(
                respData
            );
        }
    };

    respData.data = Some(response);
    respData.message = "Ok".to_string();
    respData.server_message = None;
    return HttpResponse::Ok().json(
        respData
    )
}
