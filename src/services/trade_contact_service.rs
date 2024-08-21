

use std::error::Error;

use futures::StreamExt;
use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;

use crate::models::{buy_order::BuyOrder, payment_method::{ PaymentMethodData}, sell_order::SellOrder};



pub const TRADE_CONTACT_COLLECTION:&str = "TradeContact";

pub struct  TradeContactService{

}

impl TradeContactService {

    
    // pub async fn create_trade_contact(db:&Database, bank_payment:&PaymentMethodData)->Result<InsertOneResult, Box<dyn Error>>{
    //     // Get a handle to a collection in the database.
    //     let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
  
    //     let res_sell_order =collection.insert_one(bank_payment).await;

    //     let res_order = match res_sell_order {
    //         Ok(data)=>{data},
    //         Err(err)=>{return Err(err.into())}
    //     };
    //     Ok(res_order)
    // }
}