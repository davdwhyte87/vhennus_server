use std::{ str::FromStr};

use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
use actix_web_validator::Json;
use bigdecimal::BigDecimal;
use futures::FutureExt;
use mongodb::{bson::doc, error::Error, Client, ClientSession};
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{buy_order::BuyOrder, response::{ GenericResp, Response}, sell_order::{Currency, SellOrder}}, req_models::create_sell_order_req::{CreateBuyOrderReq, CreateSellOrderReq}, services::{buy_order_service::{BuyOrderService, BUY_ORDER_COLLECTION}, mongo_service::{MongoService, DB_NAME}, sell_order_service::{SellOrderService, SELL_ORDER_COLLECTION}}, utils::auth::Claims};




#[post("/")]
pub async fn create_buy_order(

    database:Data<MongoService>,
    new_order:Json<CreateBuyOrderReq>,
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

    // get sell order
    let sell_order_c = SellOrderService::get_sell_order_by_id(&database.db, new_order.sell_order_id.to_owned()).await;
    let sell_order_c =match  sell_order_c {
        Ok(data)=>{data},
        Err(err)=>{
            return HttpResponse::InternalServerError().json(
                GenericResp::<String>{
                    message:"Error getting sell order".to_string(),
                    data: err.to_string()
                }
            )
        }
    };

    // make sure request amount is good fit for the sell order

    if new_order.amount > sell_order_c.amount {
        return HttpResponse::BadRequest().json(
            GenericResp::<String>{
                message:"Buy order is larger than sell order".to_string(),
                data: "".to_string()
            }
        )
    }
    if new_order.amount > sell_order_c.max_amount || new_order.amount< sell_order_c.min_amount{
        return HttpResponse::BadRequest().json(
            GenericResp::<String>{
                message:"buy order is larger or smaller than sell order".to_string(),
                data: "".to_string()
            }
        )  
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
            return HttpResponse::InternalServerError().json(
                GenericResp::<String>{
                    message:"Error creating buy order".to_string(),
                    data: err.to_string()
                }
            )
        }
    }

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
            return HttpResponse::InternalServerError().json(
                GenericResp::<String>{
                    message:"Error creating buy order".to_string(),
                    data: e.to_string()
                }
            )
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

    

    return HttpResponse::Ok().json(GenericResp::<BuyOrder>{
        message:"Successfully created".to_string(),
        data: buy_order.clone()
})

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
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                return HttpResponse::Unauthorized()
                    .json(Response{message:"Not authorized".to_string()})
            }
        };


        let orders = match BuyOrderService::get_all_buy_order_by_username(&database.db, claim.user_name.clone()).await{
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

        return HttpResponse::Ok().json(GenericResp::<Vec<BuyOrder>>{
            message:"Successfully created".to_string(),
            data: orders
        })

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
        // get claim 
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                return HttpResponse::Unauthorized()
                    .json(Response{message:"Not authorized".to_string()})
            }
        };

        // get buy otder by id
        let order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
            Ok(data)=>{
                match data {
                    Some(data)=>{data},
                    None=>{
                        return HttpResponse::BadRequest().json(GenericResp::<String>{
                            message:"No data found".to_string(),
                            data: "".to_string()
                        }) 
                    }
                }
            },
            Err(err)=>{
                return HttpResponse::BadRequest().json(GenericResp::<String>{
                    message:"Error getting buy order".to_string(),
                    data: err.to_string()
                }) 
            }
        };

        // check if logged in user owns the data
        if (order.user_name != claim.user_name){
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Unauthorized".to_string(),
                data: "Unauthorized".to_string()
            })  
        }

        return HttpResponse::Ok().json(GenericResp::<BuyOrder>{
            message:"Successfully created".to_string(),
            data: order
        })

}

#[get("/{id}/buyer_confirmed")]
pub async fn buyer_confirmed(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>
)->HttpResponse
{


          // get claim 
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                return HttpResponse::Unauthorized()
                    .json(Response{message:"Not authorized".to_string()})
            }
        };

    // get buy order 

    let mut buy_order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    return HttpResponse::NotFound().json(GenericResp::<String>{
                        message:"Could not find order".to_string(),
                        data: "".to_string()
                    })    
                }
            }
        },
        Err(err)=>{
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Error getting data".to_string(),
                data: "".to_string()
            }) 
        }
    };

    // make sure user owns order
    if buy_order.user_name != claim.user_name{
        return HttpResponse::Unauthorized().json(GenericResp::<String>{
            message:"Unauthorized".to_string(),
            data: "".to_string()
        })  
    }
    // modify order 
    buy_order.is_buyer_confirmed = true;

    // update database
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Error saving data".to_string(),
                data: err.to_string()
            }) 
        }
    }




    return HttpResponse::Ok().json(GenericResp::<String>{
        message:"Successfully updated".to_string(),
        data: "".to_string()
    })
}



#[get("/{id}/seller_confirmed")]
pub async fn seller_confirmed(

    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>
)->HttpResponse
{


          // get claim 
        let claim = match claim {
            Some(claim)=>{claim},
            None=>{
                return HttpResponse::Unauthorized()
                    .json(Response{message:"Not authorized".to_string()})
            }
        };

    // get buy order 

    let mut buy_order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await{
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    return HttpResponse::NotFound().json(GenericResp::<String>{
                        message:"Could not find order".to_string(),
                        data: "".to_string()
                    })    
                }
            }
        },
        Err(err)=>{
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Error getting data".to_string(),
                data: "".to_string()
            }) 
        }
    };

    // get sell order 
    let sell_order =match  SellOrderService::get_sell_order_by_id(&database.db, buy_order.sell_order_id.to_owned()).await{
        Ok(data)=>{data},
        Err(err)=>{
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Error getting data".to_string(),
                data: err.to_string()
            })   
        }
    };

    // make sure user owns sell order
    if sell_order.user_name != claim.user_name{
        return HttpResponse::Unauthorized().json(GenericResp::<String>{
            message:"Unauthorized".to_string(),
            data: "".to_string()
        })  
    }
    // modify order 
    buy_order.is_seller_confirmed = true;

    // update database
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_)=>{},
        Err(err)=>{
            return HttpResponse::BadRequest().json(GenericResp::<String>{
                message:"Error saving data".to_string(),
                data: err.to_string()
            }) 
        }
    }




    return HttpResponse::Ok().json(GenericResp::<String>{
        message:"Successfully updated".to_string(),
        data: "".to_string()
    })
}

