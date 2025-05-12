
use std::error::Error;
use futures::{future::ok, StreamExt};
use sqlx::PgPool;
use crate::models::system::System;

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

    pub async fn get_system_data(pool:&PgPool)->Result<Option<System>, Box<dyn Error>>{
        let data = sqlx::query_as!(System, 
            "SELECT * FROM system_data WHERE id=$1 ", 1)
            .fetch_optional(pool).await?;
        Ok(data)
    }
    
    pub async fn update_system_data(pool:&PgPool, system:System)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(System,
            "UPDATE system_data 
            SET 
                ngn = COALESCE($2, ngn),
                price = COALESCE($3, price)
            WHERE id=$1
             ", 1, system.ngn, system.price).execute(pool).await?;
        
        Ok(())
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