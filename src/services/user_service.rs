use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::string::ToString;
use std::vec;
use handlebars::Handlebars;
use mongodb::{Client, Database, options::ClientOptions};
use mongodb::bson::{doc, Document};
// use mongodb::bson::extjson::de::Error;
use std::error::Error;
use mongodb::bson::oid::ObjectId;
use mongodb::results::{InsertOneResult, UpdateResult};
use r2d2_mongodb::mongodb::ErrorCode::OK;
use serde_json::{json, Value};


use crate::database::db::db::DB;
use crate::models::helper::EmailData;
use crate::models::profile::{self, Profile};
use crate::models::request_models::LoginReq;
use crate::models::user::User;
use crate::utils::general::get_current_time_stamp;
use crate::utils::send_email::{ACTIVATE_EMAIL, get_body, send_email};

use super::mongo_service::MongoService;
use super::profile_service::PROFILE_COLLECTION;

pub const USER_COLLECTION:&str = "User";

pub struct UserService{
    client: Client

}

impl UserService{
    pub async fn create_user(db:&Database, user:&User)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<User>(USER_COLLECTION);

        let code:u32= 9384;

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

        //

        let res_user =user_collection.insert_one(user).await;

        let res_user = match res_user {
            Ok(res_user)=>{res_user},
            Err(err)=>{
                match session.abort_transaction().await{
                    Ok(x)=>{},
                    Err(err)=>{log::error!("abort error {}", err)}
                };
                return Err(err.into())
            }
        };


        // create profile 
        let profile = Profile{
            id : uuid::Uuid::new_v4().to_string(),
            user_name: user.user_name.clone(), 
            bio: "".to_string(), 
            name: "".to_string(),
            occupation: "".to_string(),
            image:"".to_string(), 
            created_at: get_current_time_stamp(),
            updated_at: get_current_time_stamp(),
            friends: vec![], 
            friends_models: None,
        
        };
        let res_profile =profile_collection.insert_one(profile).await;

        match res_profile {
            Ok(res_profile)=>{res_profile},
            Err(err)=>{
                match session.abort_transaction().await{
                    Ok(x)=>{},
                    Err(err)=>{log::error!("abort error {}", err)}
                };
                return Err(err.into())
            }
        };

        session.commit_transaction().await;
        Ok(res_user)
    }



    pub async fn get_by_id(db:&Database, id:String)->Result<Option<User>, Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<User>(USER_COLLECTION);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    pub async fn get_by_email(db:&Database, email:String)->Result<Option<User>, Box<dyn Error>>{

        let filter = doc! {"email":email};
        let collection = db.collection::<User>(USER_COLLECTION);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    
    pub async fn get_by_(db:&Database, filter:Document)->Result<Option<User>, Box<dyn Error>>{

        let collection = db.collection::<User>(USER_COLLECTION);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    pub async fn update(
        db:&Database,
        email:&String,
        mut new_data:&User
    )
        ->Result<UpdateResult, Box<dyn Error>>
    {
        let filter = doc! {"email":email};
        let collection = db.collection::<User>(USER_COLLECTION);
        let new_doc = doc! {
            "$set":{
                "code":new_data.code.to_owned(),
            }
        };
        let updated_doc = collection.update_one(filter,new_doc )
            .await;

        match updated_doc {
            Ok(updated_doc)=>{return Ok(updated_doc)},
            Err(err)=>{
                return Err(err.into())
            }
        }
    }
}

