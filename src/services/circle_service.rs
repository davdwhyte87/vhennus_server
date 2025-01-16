use std::{error::Error, thread::current};

use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, Database};

use crate::models::{chat::Chat, circle::Circle, user::{self, User}};

use super::user_service::USER_COLLECTION;



pub const CIRCLE_COLLECTION:&str = "Circle";

pub struct  CircleService{

}

impl CircleService {
    pub async fn create_circle(db:&Database, circle:&Circle)->Result<(), Box<dyn Error>>{

        let collection = db.collection::<Circle>(CIRCLE_COLLECTION);

        match collection.find_one(doc! {"name":circle.name.clone()}).await{
            Ok(data)=>{

                match data{
                    Some(_)=>{
                        return Err(Box::from("Circle already exists"))
                    },
                    None=>{}
                }
            },
            Err(err)=>{
                log::error!(" error getting circle  {}", err.to_string());
                return Err(err.into())
            }
        }
        match collection.insert_one(circle).await{
            Ok(_)=>{},
            Err(err)=>{
                log::error!(" error creating circle  {}", err.to_string());
                return Err(err.into())
            }
        }
        return Ok(())
    }


    pub async fn get_circle(db:&Database, name:String)->Result<Option<Circle>, Box<dyn Error>>{

        let collection = db.collection::<Circle>(CIRCLE_COLLECTION);

        let circle = match collection.find_one(doc! {"name":name}).await{
            Ok(data)=>{

               data
            },
            Err(err)=>{
                log::error!(" error getting circle  {}", err.to_string());
                return Err(err.into())
            }
        };

        return Ok(circle)
    }
}
