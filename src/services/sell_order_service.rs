use std::error::Error;

use futures::StreamExt;
use mongodb::{bson::{doc, from_document, Bson, Document}, results::InsertOneResult, Database};

use crate::models::{payment_method::PaymentMethod, sell_order::SellOrder};
use strum_macros::{EnumString, ToString};


pub const SELL_ORDER_COLLECTION:&str = "SellOrder";

pub struct  SellOrderService{

}

impl SellOrderService {
    pub async fn create_sell_order(db:&Database, order:&SellOrder)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
  
        let res_sell_order =collection.insert_one(order).await;

        let res_order = match res_sell_order {
            Ok(data)=>{data},
            Err(err)=>{return Err(err.into())}
        };
        Ok(res_order)
    }


    pub async fn get_all_sell_order_by_username(db:&Database, userName:String)->Result<Vec<SellOrder>, Box<dyn Error>>{
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "BuyOrder",
                  "localField": "buy_orders_id",
                  "foreignField": "id",
                  "as": "buy_orders"
               }
            };

        let lookup_1 = doc! {
            "$match":doc! {"user_name": userName}
        };

       let mut results = collection.aggregate(vec![lookup_1,lookup_2]).await?;
       let mut sell_orders:Vec<SellOrder> = Vec::new();
       while let Some(result) = results.next().await{
           let data: SellOrder= from_document(result?)?;
           sell_orders.push(data);
       }
       return Ok(sell_orders);
    }


    
    pub async fn get_sell_order_by_filter(db:&Database, filter:Document)->Result<Vec<SellOrder>, Box<dyn Error>>{
        
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "BuyOrder",
                  "localField": "buy_orders_id",
                  "foreignField": "id",
                  "as": "buy_orders"
               }
            };
        let match_1 = doc! {
            "$match":filter
        };

       let mut results = match collection.aggregate(vec![match_1]).await{
        Ok(dd)=>{dd},
        Err(err)=>{
            return Err(err.into())
        }
       };
       let mut sell_orders:Vec<SellOrder> = Vec::new();
       while let Some(result) = results.next().await{
           let data: SellOrder= match from_document(result?){
            Ok(d)=>{d},
            Err(err)=>{
                return Err(err.into())
            }
           };
           sell_orders.push(data);
       }
       return Ok(sell_orders);
    }





    pub async fn get_sell_order_by_id(db:&Database, id:String)->Result<SellOrder, Box<dyn Error>>{
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let filter = doc! {"id":id};
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "BuyOrder",
                  "localField": "buy_orders_id",
                  "foreignField": "id",
                  "as": "buy_orders"
               }
        };
        let lookup_3 = doc! {
            "$lookup":
               {
                  "from": "PaymentMethodData",
                  "localField": "payment_method_id",
                  "foreignField": "id",
                  "as": "payment_method_data"
               }
        };
        let pipeline = vec![
            doc! { "$match": filter },
            lookup_2,
            lookup_3,
            doc! {
                 "$unwind": "$payment_method_data"
            }
        ];

        let mut sell_orders:Vec<SellOrder> = Vec::new();
        let mut cursor =  collection.aggregate(pipeline).await;
        match cursor {
            Ok(mut cursor)=>{
                if let Some(result) = cursor.next().await{
                    match result {
                        Ok(res)=>{
                            let data: SellOrder= from_document(res)?;
                            return Ok(data);
                        },
                        Err(err)=>{
                            return Err(err.into()) ;
                        }
                    };
                }else{
                    return Err(Box::from("Error getting data")) ;
                }
            },
            Err(err)=>{
                return Err(err.into()) ;
            }
        };
  
        // match order {
        //     Ok(mut data)=>{
                
            
        //             //sell_orders.push(data);
        //         };
        //         // return Ok(sell_orders[0].clone());
        //     }, 
        //     Err(err)=>{
        //         return Err(err.into())
        //     }
        // };


        
    }


    pub async  fn update(db:&Database, sell_order:&SellOrder)->Result<(), Box<dyn Error>>{
        let filter = doc! {"id":sell_order.id.clone()};
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let update_data = doc! {"$set":doc! {
            "buy_orders_id":sell_order.buy_orders_id.to_owned(),
            "amount":sell_order.amount.to_string(),
            "min_amount": sell_order.min_amount.to_string(),
            "max_amount": sell_order.max_amount.to_string(),
            "is_closed": sell_order.is_closed,
            "currency": sell_order.currency.to_string(),
            "updated_at": sell_order.updated_at.to_owned(),
            "payment_method": sell_order.payment_method.to_owned().to_string(),
            "payment_method_id": sell_order.payment_method_id.to_owned()
            }};

        let update_res = collection.update_one(filter, update_data).await;
        match update_res {
            Ok(_)=>{},
            Err(err)=>{

                return Err(err.into());
            }
        }
        Ok(())
    }
}