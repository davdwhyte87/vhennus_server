

use std::error::Error;

use futures::StreamExt;
use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;

use crate::models::{buy_order::BuyOrder, payment_method::{ PaymentMethodData}, sell_order::SellOrder};



pub const PAYMENT_METHOD_COLLECTION:&str = "PaymentMethod";

pub struct  PaymentMethodService{

}

impl PaymentMethodService {
    pub async fn create_payment_method(db:&Database, bank_payment:&PaymentMethodData)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
  
        let res_sell_order =collection.insert_one(bank_payment).await;

        let res_order = match res_sell_order {
            Ok(data)=>{data},
            Err(err)=>{return Err(err.into())}
        };
        Ok(res_order)
    }

    pub async fn get_user_payment_method_by_id(db:&Database, id:String)->Result<Option<PaymentMethodData>, Box<dyn Error>>{
        let filter = doc! {"id":id};
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.find_one(filter).await;

        match res{
            Ok(data)=>{
                return  Ok(data);

            },
            Err(err)=>{
                return Err(err.into())
            }
        }
        
    }

    pub async fn get_all_user_payment_method_data(db:&Database, user_name:String)->Result<Vec<PaymentMethodData>, Box<dyn Error>>{
        let filter = doc! {"user_name":user_name};
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.find(filter).await;
        let mut res_data: Vec<PaymentMethodData> =vec![];
        match res{
            Ok(mut data)=>{
                while let Some(result) = data.next().await{
                    let payment_method: PaymentMethodData=match  result{
                        Ok(d)=>{d},
                        Err(err)=>{return Err(err.into())}
                    };
                    res_data.push(payment_method);
                }
                return  Ok(res_data);

            },
            Err(err)=>{
                return Err(err.into())
            }
        }
        
    }

    pub async fn update_user_payment_method(db:&Database,user_name:String, payment_method:PaymentMethodData)->Result<(), Box<dyn Error>>{
        let filter = doc! {"user_name":user_name};
        let update_data = doc! {
            "$set": doc! {
                "account_name": payment_method.account_name.to_owned(),
                "account_number": payment_method.account_number.to_owned(),
                "bank_name": payment_method.bank_name.to_owned(),
                "other": payment_method.other.to_owned(),
                "paypal_email": payment_method.paypal_email.to_owned(),
                "venmo_username": payment_method.venmo_username.to_owned(),
                "skril_email":payment_method.skrill_email.to_owned()
            }
        };
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.update_one(filter, update_data).await;

        match res{
            Ok(data)=>{
                return  Ok(());

            },
            Err(err)=>{
                return Err(err.into())
            }
        }
        
    }

    
    pub async fn delete_user_payment_method(db:&Database,id:String)->Result<(), Box<dyn Error>>{
        let filter = doc! {"id":id};
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.delete_one(filter).await;

        match res{
            Ok(data)=>{
                return  Ok(());

            },
            Err(err)=>{
                return Err(err.into())
            }
        }
        
    }
}