
use std::error::Error;

use futures::{future::OkInto, StreamExt};
use mongodb::{bson::{doc, from_document}, results::{InsertOneResult, UpdateResult}, Database};
use r2d2_mongodb::mongodb::coll;

use crate::models::{buy_order::BuyOrder, profile::Profile, sell_order::SellOrder};


pub const PROFILE_COLLECTION:&str = "Profile";

pub struct  ProfileService{

}

impl ProfileService {

    pub async fn get_profile(db:&Database, user_name:String)->Result<Profile, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<Profile>(PROFILE_COLLECTION);

        let profile = match collection.find_one(doc! {"user_name": user_name}).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error fetching profile data  {}", err.to_string());
                return Err(err.into())   
            }
        };


        match profile {
            Some(data)=>{return Ok(data)},
            None=>{
                return Ok(Profile::default())
            }
        }
    }

    pub async fn update_profile(db:&Database, profile:&Profile)->Result<UpdateResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<Profile>(PROFILE_COLLECTION);


        // get profile
        
        let ex_profile = match collection.find_one(doc! {"user_name":profile.user_name.clone()}).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error fetching profile data  {}", err.to_string());
                return Err(err.into())   
            }
        };

        match ex_profile{
            Some(data)=>{},
            None => {
                // if  no profile exists, create one
                
                match collection.insert_one(profile).await{
                    Ok(data)=>{data},
                    Err(err)=>{
                        log::error!(" error inserting profile data  {}", err.to_string());
                        return Err(err.into())   
                    }
                };
            }
        };

        // update profile
        let update_data = doc! {"$set":doc! {
            "bio":profile.bio.to_owned().clone(),
            "work":profile.work.clone(),
            "occupation": profile.occupation.clone(),
            "image": profile.image.clone(),
            "updated_at": chrono::offset::Utc::now().to_string(),
           
        }};
        let res =collection.update_one(doc! {"user_name":profile.user_name.clone()}, update_data).await;

        let res = match res {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error updating into db  {}", err.to_string());
                return Err(err.into())
            }
        };
        Ok(res)
    }
}