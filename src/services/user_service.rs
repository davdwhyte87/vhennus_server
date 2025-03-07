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

use mongodb::bson::oid::ObjectId;
use mongodb::results::{InsertOneResult, UpdateResult};

use r2d2_mongodb::mongodb::ErrorCode::{UserNotFound, OK};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use crate::database::db::db::DB;

use crate::models::helper::EmailData;

use crate::models::request_models::LoginReq;
use crate::models::user::User;

use crate::utils::general::get_current_time_stamp;
// use crate::utils::send_email::{ACTIVATE_EMAIL, get_body, send_email};

use super::mongo_service::MongoService;
use super::profile_service::PROFILE_COLLECTION;

use uuid::Uuid;

pub const USER_COLLECTION:&str = "User";

pub struct UserService{
    client: Client

}

pub struct UserView{
    
}

impl UserService{
    pub async fn create_user(pool:&PgPool, user:User)->Result<(), Box<dyn Error>>{
        let mut tx: Transaction<'_, Postgres> = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Failed to start transaction".into()),
        };
        // check if the user exists
        let user_insert = sqlx::query!(
            "INSERT INTO users (id, user_name, email, password_hash) 
             VALUES ($1, $2, $3, $4)",
            user.id.clone(),
            user.user_name.clone(),
            user.email.clone(),
            user.password_hash.clone(),
        )
            .execute(&mut *tx)
            .await;
        
        if user_insert.is_err(){
            let _ = tx.rollback().await;
            return Err(user_insert.err().unwrap().description().into());
        }
        let profile_insert = sqlx::query!(
            "INSERT INTO profiles (id,user_name) 
             VALUES ($1, $2)",
            Uuid::new_v4().to_string(),
            user.user_name.clone(),
            
        )
            .execute(&mut *tx)
            .await;

        if profile_insert.is_err() {
            let _ = tx.rollback().await;
            return Err("Failed to create profile".into());
        }
        if let Err(err) = tx.commit().await {
            log::error!("Failed to commit transaction: {}", err);
            return Err("Transaction commit failed".into());
        }
        return Ok(());
    }



    // pub async fn get_by_id(db:&Database, id:String)->Result<Option<User>, Box<dyn Error>>{
    //     let object_id = ObjectId::parse_str(id).unwrap();
    //     let filter = doc! {"_id":object_id};
    //     let collection = db.collection::<User>(USER_COLLECTION);
    //     let user_detail = collection.find_one(filter).await;
    //     match user_detail {
    //         Ok(user_detail)=>{return Ok(user_detail)},
    //         Err(err)=>{return Err(err.into())}
    //     };
    // }
    

    pub async fn get_by_username(pool:&PgPool, username:String)->Result<Option<User>, Box<dyn Error>>{
        let user =match  sqlx::query_as!(User, 
        "SELECT * FROM users WHERE user_name = $1 ", username.clone() )
            .fetch_optional(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(user);
    }
    
    
    pub async fn delete_user(pool:&PgPool, username:String)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(User,
        "UPDATE users SET is_deleted=$1 WHERE user_name = $2 ",true, username.clone() )
            .execute(pool).await?;
        
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

