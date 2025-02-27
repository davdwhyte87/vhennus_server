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
use actix_web::HttpResponse;
use diesel::{Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use diesel::r2d2::ConnectionManager;
use mongodb::bson::oid::ObjectId;
use mongodb::results::{InsertOneResult, UpdateResult};
use r2d2::PooledConnection;
use r2d2_mongodb::mongodb::ErrorCode::{UserNotFound, OK};
use serde_json::{json, Value};


use crate::database::db::db::DB;
use crate::DbPool;
use crate::models::helper::EmailData;
use crate::models::profile::{self, Profile};
use crate::models::request_models::LoginReq;
use crate::models::user::User;
use crate::schema::users::dsl::users;
use crate::schema::users::{email, user_name};
use crate::utils::general::get_current_time_stamp;
// use crate::utils::send_email::{ACTIVATE_EMAIL, get_body, send_email};

use super::mongo_service::MongoService;
use super::profile_service::PROFILE_COLLECTION;
use diesel::result::Error as DieselError;
use uuid::Uuid;

pub const USER_COLLECTION:&str = "User";

pub struct UserService{
    client: Client

}

impl UserService{
    pub async fn create_user(pool:&DbPool, user:&User)->Result<(), Box<dyn Error>>{
        let xpool = pool.clone();
        let xuser = user.clone();
        // check if the user exists
        match Self::get_by_username(pool, user.user_name.clone()).await{
            Ok(data)=>{
                return Err(Box::from("USER_EXISTS"));
            },
            Err(err)=>{
                match err.as_ref().downcast_ref::<DieselError>(){
                    Some(DieselError::NotFound)=>{
                        // this is ok... 
                    },
                    e=>{
                        log::error!("error getting user {}", e.expect("REASON").to_string());
                        return Err(Box::from("Error getting user"));
                    }
                }
            }
        }
        
        let result = actix_web::web::block( move || {
            let  conn = &mut xpool.get().expect("Couldn't get DB connection");

            // start transaction 
            let result: Result<(), DieselError> = conn
                .transaction(|conn| {
                    let inserted_user = match diesel::insert_into(crate::schema::users::table)
                        .values(&xuser)
                        .execute(conn) {
                        Ok(_)=>{},
                        Err(err)=>{
                            log::error!("error inserting user {}", err);
                            return Err(err);
                        }
                    };

                    // create profile 
                    let new_profile = Profile{
                        id: Uuid::new_v4().to_string(),
                        user_name: xuser.user_name.clone(),
                        bio: None,
                        name: None,
                        image: None,
                        created_at: Default::default(),
                        updated_at: Default::default(),
                        app_f_token: None,
                    };

                    match diesel::insert_into(crate::schema::profiles::table)
                        .values(&new_profile)
                        .execute(conn){
                        Ok(_)=>{},
                        Err(err)=>{
                            log::error!("error inserting profile {}", err);
                            return Err(err);
                        }
                    }
                    return Ok(())
                });
           
        }).await;
 
        match result{
            Ok(_)=>{},
            Err(err)=>{
                log::error!("error creating  user transaction.. {}", err);
                return Err(err.into());
            }
        };
        
        return Ok(());
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
    

    pub async fn get_by_username(pool:&DbPool, username:String)->Result<User, Box<dyn Error>>{
        let xpool = pool.clone();
        let res = actix_web::web::block( move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            let data = match users.filter(user_name.eq(&username)).first::<User>(&mut conn){
                Ok(user)=>{user},
                Err(err)=>{
                    return Err(Box::new(err));
                },
            };
            return Ok(data);
        }).await??;
        return Ok(res);
    }
    
    
    pub async fn delete_user(pool:&DbPool, username:String)->Result<(), Box<dyn Error>>{
        let xpool = pool.clone();
        
        let res = actix_web::web::block( move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            
            diesel::delete(users.filter(user_name.eq(&username))).execute(&mut conn)
        }).await??;
        Ok(())
    }

    
    // pub async fn get_by_(db:&Database, filter:Document)->Result<Option<User>, Box<dyn Error>>{
    // 
    //     let collection = db.collection::<User>(USER_COLLECTION);
    //     let user_detail = collection.find_one(filter).await;
    //     match user_detail {
    //         Ok(user_detail)=>{return Ok(user_detail)},
    //         Err(err)=>{return Err(err.into())}
    //     };
    // }

//     pub async fn update(
//         db:&Database,
//         email:&String,
//         mut new_data:&User
//     )
//         ->Result<UpdateResult, Box<dyn Error>>
//     {
//         let filter = doc! {"email":email};
//         let collection = db.collection::<User>(USER_COLLECTION);
//         let new_doc = doc! {
//             "$set":{
//                 "code":new_data.code.to_owned(),
//             }
//         };
//         let updated_doc = collection.update_one(filter,new_doc )
//             .await;
// 
//         match updated_doc {
//             Ok(updated_doc)=>{return Ok(updated_doc)},
//             Err(err)=>{
//                 return Err(err.into())
//             }
//         }
//     }
}

