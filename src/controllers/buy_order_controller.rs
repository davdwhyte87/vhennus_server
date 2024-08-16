use std::{ str::FromStr};

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use futures::FutureExt;
use mongodb::{bson::doc, error::Error, Client, ClientSession};
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{buy_order::BuyOrder, response::{ GenericResp, Response}, sell_order::{Currency, SellOrder}}, req_models::create_sell_order_req::{CreateBuyOrderReq, CreateSellOrderReq}, services::{buy_order_service::{BuyOrderService, BUY_ORDER_COLLECTION}, mongo_service::{MongoService, DB_NAME}, sell_order_service::{SellOrderService, SELL_ORDER_COLLECTION}}, utils::auth::Claims};




#[post("/buy")]
pub async fn create_buy_order(

    database:Data<MongoService>,
    new_order:Json<CreateBuyOrderReq>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    println!("new req");
    let mut respData = GenericResp::<BuyOrder>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
  
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized()
                .json(respData)
        }
    };

    // amount bigdeci 
    // let amount = BigDecimal::from_str(new_order.amount);

    // get sell order
    let sell_order_c = SellOrderService::get_sell_order_by_id(&database.db, new_order.sell_order_id.to_owned()).await;
    let sell_order_c =match  sell_order_c {
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting  sell order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::InternalServerError().json(
           respData
            )
        }
    };

    // make sure request amount is good fit for the sell order

    if new_order.amount > sell_order_c.amount {
        respData.message = "Buy order is larger than sell order".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::BadRequest().json(
          respData
        )
    }
    if new_order.amount > sell_order_c.max_amount || new_order.amount< sell_order_c.min_amount{
        respData.message = "buy order is larger or smaller than sell order".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::BadRequest().json(respData)  
    }

    // create buy order
    let mut buy_order = BuyOrder{
        id: Uuid::new_v4().to_string(),
        user_name: claim.user_name.clone(),
        is_buyer_confirmed:false,
        amount: new_order.amount.to_owned(),
        sell_order_id: new_order.sell_order_id.to_owned(),
        is_seller_confirmed:false,
        is_canceled:false,
        is_reported:false,
        created_at: chrono::offset::Utc::now().to_string(),
        updated_at: chrono::offset::Utc::now().to_string(),
    };

    let seller_order_id = new_order.sell_order_id.to_owned();

    let mut session = database.client.start_session().await.unwrap();
   
    match session
    .start_transaction().await{
        Ok(_)=>{},
        Err(err)=>{
            respData.message = "error creating buy order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(
              respData
            )
        }
    }

    // start transaction
    let result = async {
        create_buy_order_update_sell_order(&mut session, buy_order.clone(), seller_order_id.clone(), sell_order_c.amount - new_order.amount.clone()).await
    }.await;


    match result {
        Ok(_) => {
            session.commit_transaction().await;
            println!("Transaction succeeded");
        },
        Err(e) => {
            session.abort_transaction().await;
            respData.message = "error creating buy order".to_string();
            respData.data = None;
            respData.server_message = Some(e.to_string());
            return HttpResponse::InternalServerError().json(respData)
        },
    }



    

        // Ok(_)=>{},
        // Err(err)=>{
        //     return HttpResponse::InternalServerError().json(
        //         GenericResp::<String>{
        //             message:"Error creating buy order".to_string(),
        //             data: err.to_string()
        //         }
        //     )
        // }
    


    // // save order
    // match BuyOrderService::create_buy_order(&database.db, &buy_order).await{
    //     Ok(_)=>{
    //         // update the sell order

    //         SellOrderService::get_all_sell_order_by_id(&database.db, new_order.sell_order_id.to_owned())
    //     },
    //     Err(err)=>{
    //         return HttpResponse::InternalServerError().json(
    //             GenericResp{
    //                 message:"Error ".to_string(),
    //                 data: err.to_string()
    //             }
    //         )  
    //     }
    // };

    

    respData.message = "Ok".to_string();
    respData.data = Some(buy_order.clone());
    respData.server_message = None;

    return HttpResponse::Ok().json(respData)

}


async fn  create_buy_order_update_sell_order(
    session: &mut ClientSession,
     mut buy_order: BuyOrder,
      sell_order_id:String,
      new_amount:BigDecimal
)->Result<(), Error>{


    let sell_order_collection = session.client().database(DB_NAME).collection::<SellOrder>(SELL_ORDER_COLLECTION);
    let buy_order_collection = session.client().database(DB_NAME).collection::<BuyOrder>(BUY_ORDER_COLLECTION);

    let buy_order_id = buy_order.id.to_owned();
    match buy_order_collection.insert_one(buy_order).await {
        Ok(_)=>{},
        Err(err)=>{
            return Err(err.into())
        }
    };

    // get sell order 
    let sell_order_id_2 = sell_order_id.clone();
    let filter = doc! {"id":sell_order_id};
    let order =  sell_order_collection.find_one(filter).await;
    let mut order_data = match order {
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    return Err(Error::custom("No data found".to_string()))
                }
            }
        }, 
        Err(err)=>{
            return Err(err.into())
        }
    };

    // update sell order 
   order_data.buy_orders_id.push(buy_order_id);

   //save sell data 
   let update_filter = doc! {"id": sell_order_id_2};
   let update_data = doc! {"$set":doc! {"buy_orders_id":order_data.buy_orders_id, "amount":new_amount.to_string()}};
   match sell_order_collection.update_one(update_filter, update_data).await{
        Ok(_)=>{},
        Err(err)=>{
            return Err(err.into())
        }
   };

    Ok(())
}



#[get("/my_orders")]
pub async fn get_my_buy_orders(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
    )->HttpResponse
    {

        let mut respData = GenericResp::<Vec<BuyOrder>>{
            message:"".to_string(),
            server_message: Some("".to_string()),
            data: None
        };
      
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                respData.message = "Unauthorized".to_string();
                respData.data = None;
                respData.server_message = None;
                return HttpResponse::Unauthorized()
                    .json(respData)
            }
        };


        let orders = match BuyOrderService::get_all_buy_order_by_username(&database.db, claim.user_name.clone()).await{
            Ok(data)=>{data},
            Err(err)=>{
                respData.message = "error getting buy order".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::InternalServerError().json(
                  respData
                )  
            }
        };

        respData.message = "ok".to_string();
        respData.data = Some(orders);
        respData.server_message = None;
        return HttpResponse::Ok().json(respData)

}

#[derive(Deserialize)]
struct GetBuyOrderPath {
    id: String,
   
}


#[get("/{id}")]
pub async fn get_single_buy_order(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>
    )->HttpResponse{

        let mut respData = GenericResp::<BuyOrder>{
            message:"".to_string(),
            server_message: Some("".to_string()),
            data: None
        };
        // get claim 
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                respData.message = "Unauthorized".to_string();
                respData.data = None;
                respData.server_message = None;
                return HttpResponse::Unauthorized()
                    .json(respData)
            }
        };

        // get buy otder by id
        let order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
            Ok(data)=>{
                match data {
                    Some(data)=>{data},
                    None=>{
                        respData.message = "No data found".to_string();
                        respData.data =None;
                        respData.server_message = None;
                        return HttpResponse::BadRequest().json(respData) 
                    }
                }
            },
            Err(err)=>{
                respData.message = "Error getting buy order".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::BadRequest().json(
                    respData
                ) 
            }
        };

        // check if logged in user owns the data
        // if (order.user_name != claim.user_name){
        //     return HttpResponse::BadRequest().json(GenericResp::<String>{
        //         message:"Unauthorized".to_string(),
        //         data: "Unauthorized".to_string()
        //     })  
        // }


        respData.message = "ok".to_string();
        respData.data = Some(order);
        respData.server_message = None;
        return HttpResponse::Ok().json(respData)

}

#[get("/{id}/buyer_confirmed")]
pub async fn buyer_confirmed(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>
)->HttpResponse
{

    
    let mut respData = GenericResp::<BuyOrder>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
        // get claim 
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            
        respData.message = "Unauthorized".to_string();
        respData.data = None;
        respData.server_message = None;
            return HttpResponse::Unauthorized()
                .json(respData)
        }
    };

    // get buy order 

    let mut buy_order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    
                    respData.message = "Could not find order".to_string();
                    respData.data = None;
                    respData.server_message = None;
                    return HttpResponse::NotFound().json(respData)    
                }
            }
        },
        Err(err)=>{
            
        respData.message = "Error getting buy order".to_string();
        respData.data = None;
        respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(
                respData
            ) 
        }
    };

    // make sure user owns order
    if buy_order.user_name != claim.user_name{
        respData.message = "Unauthorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData)  
    }
    // modify order 
    buy_order.is_buyer_confirmed = true;

    // update database
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_)=>{},
        Err(err)=>{
            respData.message = "Error updating buy order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData) 
        }
    }


    respData.message = "Buy order confirmed".to_string();
    respData.data = None;
    respData.server_message = None;
    return HttpResponse::Ok().json(respData)
}



#[get("/{id}/seller_confirmed")]
pub async fn seller_confirmed(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>
)->HttpResponse
{
    let mut respData = GenericResp::<BuyOrder>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

        // get claim 
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized()
                .json(respData)
        }
    };

    // get buy order 

    let mut buy_order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    respData.message = "Could not find order".to_string();
                    respData.data = None;
                    respData.server_message = None;
                    return HttpResponse::NotFound().json(respData)    
                }
            }
        },
        Err(err)=>{
            respData.message = "Error getting data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData) 
        }
    };

    // get sell order 
    let sell_order =match  SellOrderService::get_sell_order_by_id(&database.db, buy_order.sell_order_id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData)   
        }
    };

    // make sure user owns sell order
    if sell_order.user_name != claim.user_name{
        respData.message = "Unauthorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData)  
    }
    // modify order 
    buy_order.is_seller_confirmed = true;

    // update database
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_)=>{},
        Err(err)=>{
            respData.message = "Error saving data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData) 
        }
    }

    // if 
    


    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;

    return HttpResponse::Ok().json(respData)
}



#[get("/{id}/cancel")]
pub async fn cancel_buy_order(
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>
)->HttpResponse{
    let mut respData = GenericResp::<BuyOrder>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    // get claim 
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

    // get buy order 

    let mut buy_order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    respData.message = "Could not find data".to_string();
                    respData.data = None;
                    respData.server_message = None;
                    return HttpResponse::NotFound().json(respData)    
                }
            }
        },
        Err(err)=>{
            respData.message = "Error getting data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData) 
        }
    };

    // make sure request user is the order owner
    if buy_order.user_name != claim.user_name{
        respData.message = "Not authorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData) 
    }

    // update order
    buy_order.is_canceled = true;

    // update dataase
    match BuyOrderService::update(&database.db, &buy_order).await{
        Ok(_)=>{},
        Err(err)=>{
            respData.message = "Error updating order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData)   
        }
    }


    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;
    return HttpResponse::Ok().json(respData)
}

