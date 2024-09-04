

use std::error::Error;

use futures::StreamExt;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};

use crate::models::{buy_order::BuyOrder, sell_order::SellOrder};



pub const BUY_ORDER_COLLECTION:&str = "BuyOrder";

pub struct  BuyOrderService{

}

impl BuyOrderService {
    pub async fn create_buy_order(db:&Database, order:&BuyOrder)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
  
        let res_sell_order =collection.insert_one(order).await;

        let res_order = match res_sell_order {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting into db  {}", err.to_string());
                return Err(err.into())
            }
        };
        Ok(res_order)
    }


    pub async fn get_all_buy_order_by_username(db:&Database, userName:String)->Result<Vec<BuyOrder>, Box<dyn Error>>{
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
       let mut results = collection.find(doc! {"user_name":userName}).await?;
       let mut buy_orders:Vec<BuyOrder> = Vec::new();
       while let Some(result) = results.next().await{
           let data= result.unwrap();
           buy_orders.push(data);
       }
       return Ok(buy_orders);
    }

    pub async fn get_single_order_by_id(db:&Database, id:String)->Result<Option<BuyOrder>, Box<dyn Error>>{
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
        let  results = collection.find_one(doc! {"id":id}).await;

        match  results {
            Ok(data)=>{return Ok(data)},
            Err(err)=>{
                log::error!(" error getting data from db  {}", err.to_string());
                return Err(err.into())
            }
        }
    }

    
    pub async  fn update(db:&Database, buy_order:&BuyOrder)->Result<(), Box<dyn Error>>{
        let filter = doc! {"id":buy_order.id.clone()};
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
        let update_data = doc! {"$set":doc! {
            "amount":buy_order.amount.to_owned().to_string(),
            "is_seller_confirmed":buy_order.is_seller_confirmed,
            "is_buyer_confirmed": buy_order.is_buyer_confirmed,
            "is_canceled": buy_order.is_canceled,
            "is_reported": buy_order.is_reported,
            "updated_at": chrono::offset::Utc::now().to_string(),
           
            }};

        let update_res = collection.update_one(filter, update_data).await;
        match update_res {
            Ok(_)=>{},
            Err(err)=>{
                log::error!(" error updating db {}", err.to_string());
                return Err(err.into());
            }
        }
        Ok(())
    }

    
}