use std::str::FromStr;

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{payment_method::PaymentMethod, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}}, req_models::create_sell_order_req::{CreateSellOrderReq, UpdateSellOrderReq}, services::{mongo_service::MongoService, sell_order_service::SellOrderService}, utils::auth::Claims};




#[post("/")]
pub async fn create_sell_order(

    database:Data<MongoService>,
    new_order:Json<CreateSellOrderReq>,
    claim:Option<ReqData<Claims>>
    )->HttpResponse{
        let mut respData = GenericResp::<SellOrder>{
            message:"".to_string(),
            server_message: Some("".to_string()),
            data: Some(SellOrder::default())
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

    // amount bigdeci 
    // let amount = BigDecimal::from_str(new_order.amount);
    
    let sell_order = SellOrder{
        id: Uuid::new_v4().to_string(),
        user_name: claim.user_name.clone(),
        buy_orders_id : vec![],
        buy_orders : None, 
        amount : new_order.amount.to_owned(),
        min_amount: new_order.min_amount.to_owned(),
        max_amount: new_order.amount.to_owned(),
        is_closed: false,
        created_at: chrono::offset::Utc::now().to_string(),
        currency: new_order.currency.to_owned(),
        updated_at: Some(chrono::offset::Utc::now().to_string()),
        payment_method: new_order.payment_method.to_owned(),
        payment_method_id: new_order.payment_method_id.to_owned(),
        payment_method_data: None
    };

    // save order
    match SellOrderService::create_sell_order(&database.db, &sell_order).await{

        Ok(_)=>{},
        Err(err)=>{ 
            respData.message = "Error creating ".to_string();
            respData.server_message = Some(err.to_string());
            respData.data =None;
            return HttpResponse::InternalServerError().json(
              respData
               
            )  
        }
    };


    respData.data = None;
    respData.message = "Created".to_string();
    respData.server_message = None;
    return HttpResponse::Ok().json(
        respData
    )

}




#[get("/my_orders")]
pub async fn get_my_sell_orders(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
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
                respData.data = None;
                respData.message = "Unauthorized".to_string();
                respData.server_message = None;
                return HttpResponse::Unauthorized()
                    .json(
                        respData
                    )
            }
        };


        let orders = match SellOrderService::get_all_sell_order_by_username(&database.db, claim.user_name.clone()).await{
            Ok(data)=>{data},
            Err(err)=>{
                respData.data = None;
                respData.server_message = Some(err.to_string());
                respData.message = "Error getting sell order".to_string();

                return HttpResponse::InternalServerError().json(
                  respData
                )  
            }
        };
        

        respData.data = Some(orders);
        respData.message = "ok".to_string();
        respData.server_message = None;

        return HttpResponse::Ok().json(
            respData
        )

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
    let mut respData = GenericResp::<SellOrder>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
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
            respData.message = "Error getting sell order".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::Ok().json(respData) 
        }
    };

    respData.data = Some(order);

    return HttpResponse::Ok().json(respData)

}



#[get("/cancel/{id}")]
pub async fn cancel_sell_order(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    inf: web::Path<GetSingleSellOrderPath>
)->HttpResponse
{
    let mut respData = GenericResp::<SellOrder>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
   // get claims 
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            return HttpResponse::Unauthorized()
                .json(Response{message:"Not authorized".to_string()})
        }
    }; 

    // get sell order 
    println!("{}", inf.id);
    let mut order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting sell order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::BadRequest().json(
                respData
            ) 
        }
    };

    // check of any of the buy orders is still active 
    match order.buy_orders.clone() {
        Some(data)=>{
            for buy_order in data {
                if !(buy_order.is_buyer_confirmed && buy_order.is_seller_confirmed) {
                    respData.message = "There is still an open buy order".to_string();
                    respData.data = None;
                    respData.server_message = None;

                    return HttpResponse::BadRequest().json(respData) 
                }
            }
        },
        None=>{

        }

    }

    order.is_closed = true;

    match SellOrderService::update(&database.db, &order).await{
        Ok(_)=>{},
        Err(err)=>{
            respData.message = "error updating sell order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::InternalServerError().json(respData)  ;
        }
    }

    respData.data = Some(order);
    respData.message = "Ok".to_string();
    respData.server_message = None;

    return HttpResponse::Ok().json(respData)
}


#[post("/update/{id}")]
pub async fn update_sell_order(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    new_order:Json<UpdateSellOrderReq>,
    inf: web::Path<GetSingleSellOrderPath>
)->HttpResponse
{
    let mut respData = GenericResp::<SellOrder>{
        message:"".to_string(),
        server_message: None,
        data: None
    };

    // get claims
    // get claims 
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

    // get sell order 
    let mut sell_order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "error getting data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData)   
        }
    };

    // check if user owns the order 
    if sell_order.user_name != claim.user_name{
        respData.message = "Unauthorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData)      
    }

    // update data 
    if new_order.currency.is_some(){
        sell_order.currency = new_order.currency.to_owned().unwrap()
    }
    if new_order.max_amount.is_some(){
        sell_order.max_amount = new_order.max_amount.to_owned().unwrap()
    }
    if new_order.min_amount.is_some(){
        sell_order.min_amount = new_order.min_amount.to_owned().unwrap()
    }

    // update on database 
    match SellOrderService::update(&database.db, &sell_order).await {
        Ok(_)=>{},
        Err(err)=>{
            respData.data = None;
            respData.server_message = Some(err.to_string());
            respData.message = "Error updating data".to_string();

            return HttpResponse::BadRequest().json(respData)   
        }
    }

    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;

    return HttpResponse::Ok().json(respData) 
}




#[get("/open_orders")]
pub async fn get_all_open_sell_orders(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
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
                respData.data = None;
                respData.message = "Unauthorized".to_string();
                respData.server_message = None;
                return HttpResponse::Unauthorized()
                    .json(
                        respData
                    )
            }
        };


        let orders = match SellOrderService::get_all_sell_order_by_username(&database.db, claim.user_name.clone()).await{
            Ok(data)=>{data},
            Err(err)=>{
                respData.data = None;
                respData.server_message = Some(err.to_string());
                respData.message = "Error getting sell order".to_string();

                return HttpResponse::InternalServerError().json(
                  respData
                )  
            }
        };
        

        respData.data = Some(orders);
        respData.message = "ok".to_string();
        respData.server_message = None;

        return HttpResponse::Ok().json(
            respData
        )

}
