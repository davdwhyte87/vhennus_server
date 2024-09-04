

use std::error::Error;

use futures::{future::ok, StreamExt};
use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;

use crate::models::{buy_order::BuyOrder, payment_method::PaymentMethodData, sell_order::SellOrder, system::System};



pub const SYSTEM_COLLECTION:&str = "System";

pub struct  SystemService{

}

impl SystemService {
    // pub async fn create_payment_method(db:&Database, bank_payment:&PaymentMethodData)->Result<InsertOneResult, Box<dyn Error>>{
    //     // Get a handle to a collection in the database.
    //     let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
  
    //     let res_sell_order =collection.insert_one(bank_payment).await;

    //     let res_order = match res_sell_order {
    //         Ok(data)=>{data},
    //         Err(err)=>{return Err(err.into())}
    //     };
    //     Ok(res_order)
    // }

    pub async fn get_system_data(db:&Database)->Result<Option<System>, Box<dyn Error>>{
        let filter = doc! {"id":"1"};
        let collection = db.collection::<System>(SYSTEM_COLLECTION);
        let res = collection.find_one(filter).await;

        match res{
            Ok(data)=>{
                
                return  Ok(data);

            },
            Err(err)=>{
                log::error!(" error getting system data  {}", err.to_string());
                return Err(err.into())
            }
        }
        
    }

    pub async  fn update_system(db:&Database, system:&System)->Result<(), Box<dyn Error>>{
        let filter = doc! {"id":"1"};
        let collection = db.collection::<System>(SYSTEM_COLLECTION);
        let changes = doc! {
            "$set": doc! {
                "price": system.price.to_string(), 
                "android_app_version": system.android_app_version.to_owned()
            }
        };
        let res = collection.update_one(filter, changes).await;

        match res {
            Ok(_)=>{ return Ok(())},
            Err(err)=>{
                log::error!(" error updating system data  {}", err.to_string());
                return Err(err.into())
            }
        }
    }
}