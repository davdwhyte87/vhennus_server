use std::str::FromStr;

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}}, req_models::create_sell_order_req::CreateSellOrderReq, services::{mongo_service::MongoService, sell_order_service::SellOrderService}, utils::auth::Claims};




#[post("/")]
pub async fn create_sell_order(

    database:Data<MongoService>,
    new_order:Json<CreateSellOrderReq>,
    claim:Option<ReqData<Claims>>
    )->HttpResponse{

    println!("new req");

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    };

    // amount bigdeci 
    // let amount = BigDecimal::from_str(new_order.amount);
    
    let sell_order = SellOrder{
        id: Uuid::new_v4().to_string(),
        user_name: claim.user_name.clone(),
        buy_orders_id : vec![],
        buy_orders : None, 
        amount : new_order.amount.to_owned(),
        min_amount: new_order.min_amount.to_owned(),
        max_amount: new_order.max_amount.to_owned(),
        is_cancelled: false,
        created_at: chrono::offset::Utc::now().to_string(),
        currency: new_order.currency.to_owned()
    };

    // save order
    match SellOrderService::create_sell_order(&database.db, &sell_order).await{

        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::InternalServerError().json(
                GenericResp{
                    message:"Successfully created".to_string(),
                    data: err.to_string()
                }
            )  
        }
    };

    return HttpResponse::Ok().json(Response{message:"Successfully created".to_string()})

}




#[get("/my_orders")]
pub async fn get_my_sell_orders(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
    )->HttpResponse
    {
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                return HttpResponse::Unauthorized()
                    .json(Response{message:"Not authorized".to_string()})
            }
        };


        let orders = match SellOrderService::get_all_sell_order_by_username(&database.db, claim.user_name.clone()).await{
            Ok(data)=>{data},
            Err(err)=>{
                return HttpResponse::InternalServerError().json(
                    GenericResp::<String>{
                        message:"Successfully created".to_string(),
                        data: err.to_string()
                    }
                )  
            }
        };

        return HttpResponse::Ok().json(GenericResp::<Vec<SellOrder>>{
            message:"Successfully created".to_string(),
            data: orders
        })

}



#[derive(Deserialize)]
struct GetSingleSellOrderPath {
    id: String,
}

#[get("/{id}")]
pub async fn get_single_sell_order(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    inf: web::Path<GetSingleSellOrderPath>
)->HttpResponse
{
    // get claims 
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    };

    // get sell order 
    let order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            return HttpResponse::Ok().json(GenericResp::<String>{
                message:"Error getting sell order".to_string(),
                data: "".to_string()
            }) 
        }
    };

    return HttpResponse::Ok().json(GenericResp::<SellOrder>{
        message:"Ok".to_string(),
        data: order
    })

}



#[get("/{id}")]
pub async fn cancel_sell_order(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    inf: web::Path<GetSingleSellOrderPath>
)->HttpResponse
{
   // get claims 
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    }; 

    // get sell order 
    
    let order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Error getting sell order".to_string(),
                data: "".to_string()
            }) 
        }
    };

    // check of any of the buy orders is still active 
    match order.buy_orders {
        Some(data)=>{
            for buy_order in data {
                if !(buy_order.is_buyer_confirmed && buy_order.is_seller_confirmed) {
                    return HttpResponse::BadRequest().json(GenericResp::<String>{
                        message:"There is still an open buy order".to_string(),
                        data: "".to_string()
                    }) 
                }
            }
        },
        None=>{

        }

    }

    SellOrderService


    return HttpResponse::Ok().json(GenericResp::<SellOrder>{
        message:"Ok".to_string(),
        data: order
    })
}