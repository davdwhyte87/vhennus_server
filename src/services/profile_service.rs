
use std::{error::Error, vec};

use futures::{future::OkInto, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document}, results::{InsertOneResult, UpdateResult}, Database};
use r2d2_mongodb::mongodb::coll;

use crate::{models::{buy_order::BuyOrder, profile::Profile, sell_order::SellOrder}, utils::general::get_current_time_stamp};


pub const PROFILE_COLLECTION:&str = "Profile";

pub struct  ProfileService{

}

impl ProfileService {
    pub async fn get_friends(db:&Database, user_name:String)->Result<Profile, Box<dyn Error>>{
        let collection = db.collection::<Profile>(PROFILE_COLLECTION);
        let lookup_3 = doc! {
            "$lookup":
               {
                  "from": "Profile",
                  "localField": "friends",
                  "foreignField": "user_name",
                  "as": "friends_models"
               }
        };

        let match_ = doc! {
            "$match":{
                "user_name": user_name.clone()
            }
        };

        let pipeline = vec![match_, lookup_3];
        let mut res = match collection.aggregate(pipeline).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error fetching profile data  {}", err.to_string());
                return Err(err.into())   
            }
        };

        let mut profiles:Vec<Profile> = vec![];
        while let Some(data) = res.try_next().await?{
            let profile:Profile = from_document(data)?;
            profiles.push(profile);
        }

        let profile = match profiles.get(0){
            Some(data)=>{
                return Ok(data.clone())
            },
            None=>{
                // create 
                let new_profile  = Profile{
                    id: uuid::Uuid::new_v4().to_string(),
                    user_name: user_name.clone(),
                    bio: "".to_string(),
                    name: "".to_string(),
                    occupation: "".to_string(),
                    created_at: get_current_time_stamp(),
                    image:"".to_string(),
                    updated_at: get_current_time_stamp(),
                    friends: vec![],
                    friends_models: None
                };

                // create profile 
                match collection.insert_one(new_profile.clone()).await{
                    Ok(_)=>{},
                    Err(err)=>{
                        log::error!(" error creating profile data  {}", err.to_string());
                        return Err(err.into()) 
                    }
                };

                new_profile
            }
        };
        
        return Ok(profile)
    }


    pub async fn search(db:&Database, data:String)->Result<Vec<Profile>, Box<dyn Error>>{
        let filter =  doc! { "user_name": { "$regex": data, "$options": "i" } };
        let collection = db.collection::<Profile>(PROFILE_COLLECTION);

        let mut cursor = match collection.find(filter).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error fetching profile data  {}", err.to_string());
                return Err(err.into())   
            }
        };
        let mut profiles:Vec<Profile> = vec![];
        while  let Some(data) = cursor.try_next().await? {  
            profiles.push(data);
        }

        Ok(profiles)
    }

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
            "name":profile.name.clone(),
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