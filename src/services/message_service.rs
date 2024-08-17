

use std::error::Error;

use futures::StreamExt;
use mongodb::{bson::{doc, from_document, Document}, results::InsertOneResult, Database};

use crate::models::{buy_order::BuyOrder, message::OrderMessage, sell_order::SellOrder};



pub const ORDER_MESSAGE_COLLECTION:&str = "OrderMessage";

pub struct  OrderMessageService{

}

impl OrderMessageService {
    pub async fn create_message(db:&Database, message:&OrderMessage)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<OrderMessage>(ORDER_MESSAGE_COLLECTION);
  
        let result =collection.insert_one(message).await;

        let data = match result {
            Ok(data)=>{data},
            Err(err)=>{return Err(err.into())}
        };
        Ok(data)
    }

    pub async fn get_message(db:&Database, filter:Document)->Result<Vec<OrderMessage>, Box<dyn Error>>{
        let collection = db.collection::<OrderMessage>(ORDER_MESSAGE_COLLECTION);
  
        let results =collection.find(filter).await;
        let mut messages:Vec<OrderMessage> = Vec::new();
        let mut results = match results{
            Ok(dd)=>{dd},
            Err(err)=>{
                return Err(err.into())
            }
        };
        
        
        while let Some(result) = results.next().await{
            let data= match result{
                Ok(data)=>{data},
                Err(err)=>{
                    return Err(err.into())
                }
            };
            messages.push(data);
        }
    
        Ok(messages)  
    }


}