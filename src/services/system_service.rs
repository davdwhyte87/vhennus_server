

use std::error::Error;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use futures::{future::ok, StreamExt};
// use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;
use crate::DbPool;
use crate::models::{buy_order::BuyOrder, payment_method::PaymentMethodData, sell_order::SellOrder, system::System};
use crate::schema::system_data::dsl::system_data;
use crate::schema::system_data::id;

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

    pub async fn get_system_data(pool:&DbPool)->Result<Option<System>, Box<dyn Error>>{
        let xpool = pool.clone();
        
        let res =actix_web::web::block(move || {
            let mut conn = xpool.get().unwrap();
            
            let data = match system_data.filter(id.eq(1)).first::<System>(&mut conn).optional(){
              Ok(data) => data,
                Err(err)=>{
                    return Err(Box::new(err));
                }
            };
            
            Ok(data)
        }).await??;
        Ok(res)
    }

    // pub async  fn update_system(db:&Database, system:&System)->Result<(), Box<dyn Error>>{
    //     let filter = doc! {"id":"1"};
    //     let collection = db.collection::<System>(SYSTEM_COLLECTION);
    //     let changes = doc! {
    //         "$set": doc! {
    //             "price": system.price.to_string(), 
    //             "android_app_version": system.android_app_version.to_owned()
    //         }
    //     };
    //     let res = collection.update_one(filter, changes).await;
    // 
    //     match res {
    //         Ok(_)=>{ return Ok(())},
    //         Err(err)=>{
    //             log::error!(" error updating system data  {}", err.to_string());
    //             return Err(err.into())
    //         }
    //     }
    // }
}