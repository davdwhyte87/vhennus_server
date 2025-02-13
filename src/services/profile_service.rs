
use std::{error::Error, vec};

use futures::{future::OkInto, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document, Regex}, results::{InsertOneResult, UpdateResult}, Database};
use r2d2_mongodb::mongodb::coll;

use crate::{models::{buy_order::BuyOrder, post::Post, profile::Profile, sell_order::SellOrder, user::User}, utils::general::get_current_time_stamp};

use super::{mongo_service::MongoService, post_service::POST_SERVICE_COLLECTION, user_service::USER_COLLECTION};


pub const PROFILE_COLLECTION:&str = "Profile";

pub struct  ProfileService{

}

impl ProfileService {
    pub async fn get_user_profile(db:&Database, user_name:String)->Result<Profile, Box<dyn Error>>{
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
                    friends_models: None,
                    app_f_token: None
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
        // let filter =  doc! { "user_name": { "$regex": data, "$options": "i" } };

        let collection = db.collection::<Profile>(PROFILE_COLLECTION);

        let trimmed_data = data.trim();
        //let escaped_data = regex::escape(trimmed_data);
        let words: Vec<&str> = trimmed_data.split_whitespace().collect();
        let regex_pattern = if words.is_empty() {
            "".to_string()
        } else {
            words
                .iter()
                .map(|word| format!("(?=.*{})", regex::escape(word))) // Lookahead assertion
                .collect::<String>()
        };
        let regex = Regex {
            pattern: regex_pattern,
            options: "i".to_string(),
        };
        let filter =doc! {
            "$or": [
                doc! { "user_name": { "$regex": &regex } },
                doc! { "name": { "$regex": &regex } },
            ]
        };

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
            "app_f_token": profile.app_f_token.clone()
           
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


    pub async  fn  delete_account(db:&Database, userName:String)->Result<(), Box<dyn Error>>{
        
        let mut  session = match db.client().start_session().await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!("error creating database session {}", err);
                return Err(err.into());
            }
        };
        match session.start_transaction().await{
            Ok(data)=>{},
            Err(err)=>{
                log::error!("error creating session transaction  {}", err);
                return Err(err.into());   
            }
        };

        let user_collection = session.client().database(&MongoService::get_db_name()).collection::<User>(USER_COLLECTION);
        let profile_collection = session.client().database(&MongoService::get_db_name()).collection::<Profile>(PROFILE_COLLECTION);
        let post_collection = session.client().database(&MongoService::get_db_name()).collection::<Post>(POST_SERVICE_COLLECTION);
        // delete all posts
        match post_collection.delete_many(doc! {"user_name": userName.clone()}).await{
            Ok(_)=>{},
            Err(err)=>{
                log::error!("error deleting all posts .. {}", err);
                match session.abort_transaction().await{
                    Ok(x)=>{},
                    Err(err)=>{log::error!("abort error {}", err)}
                };
            }
        }

        // update user document
        match user_collection.update_one(doc! {"user_name":userName.clone()},
         doc! {"$set":doc! {
            "is_deleted":true
         }}).await {
            Ok(_)=>{},
            Err(err)=>{
                log::error!("error updating user document .. {}", err);
                match session.abort_transaction().await{
                    Ok(x)=>{},
                    Err(err)=>{log::error!("abort error {}", err)}
                }; 
            }
        }

        // commit data changes
        session.commit_transaction().await;
        Ok(())
    }
}