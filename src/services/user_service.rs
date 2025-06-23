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
use rand::Rng;
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
use crate::services::email_service::EmailService;

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
        if Self::user_email_exists(pool,user.email.clone().unwrap().as_str()).await.unwrap(){
            return Err(Box::from("UserEmail already exists"));
        }


        let code = rand::thread_rng()
            .gen_range(100_000..1_000_000) ;
        //create user
        let user_insert = sqlx::query_as!(User,
            "INSERT INTO users (id, user_name, email, password_hash, code) 
             VALUES ($1, $2, $3, $4, $5)",
            user.id.clone(),
            user.user_name.clone(),
            user.email.clone(),
            user.password_hash.clone(),
            code.clone()
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
        
        // send user email
        match EmailService::send_signup_email(user.email.unwrap(), code.to_string()).await{
            Ok(email)=>{},
            Err(err)=>{
               log::error!("Failed to send signup email: {}", err);
                let _ = tx.rollback().await;
                return Err(err);
            }
        };
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

    pub async fn get_all(pool:&PgPool)->Result<Vec<User>, Box<dyn Error>>{
        let user =match  sqlx::query_as!(User,
        "SELECT * FROM users")
            .fetch_all(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(user);
    }



    pub async fn get_by_email(pool:&PgPool, email:String)->Result<Option<User>, Box<dyn Error>>{
        let user =match  sqlx::query_as!(User, 
        "SELECT * FROM users WHERE email = $1 ", email.clone() )
            .fetch_optional(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(user);
    }

    pub async fn confirm_user_email(pool:&PgPool, email:String, code:String)->Result<(), Box<dyn Error>>{
        
        // get user 
        let user =match Self::get_by_email(pool,email.clone()).await{
            Ok(user) => { 
                match user {
                    Some(user) => {user},
                    None=>{
                        return Err(Box::from("User not found"));
                    }
                }
            },
            Err(err)=>{
                log::error!("Error getting user");
                return Err(err);
            }
        };
        if code != user.code.unwrap().to_string(){
            return Err(Box::from("Wrong code"));
        }
        let res = sqlx::query_as!(User,
        "UPDATE users SET email_confirmed=$1 WHERE email = $2 ",true, email.clone() )
            .execute(pool).await?;

        Ok(())
    }
    
    pub async fn update_code(pool:&PgPool, email:String)->Result<(), Box<dyn Error>>{
        let code = rand::thread_rng()
            .gen_range(100_000..1_000_000) ;
        
        let res = sqlx::query_as!(User,
        "UPDATE users SET code=$1 WHERE email = $2 ",code.clone(), email.clone() )
            .execute(pool).await?;
        
        // send user email
        match EmailService::send_signup_email(email.clone(), code.to_string()).await{
            Ok(email)=>{},
            Err(err)=>{
                log::error!("Failed to send signup email: {}", err);
                return Err(err);
            }
        };
        Ok(())
    }

    pub async fn update(pool:&PgPool, user:User)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(User,
        "UPDATE users SET 
                 code=COALESCE($2,code),
                 password_hash = COALESCE($3, password_hash)
        WHERE user_name = $1
        ",user.user_name.clone(), user.code.clone(), user.password_hash.clone() )
            
            .execute(pool).await?;
        Ok(())
    }
    
    
    pub async fn delete_user(pool:&PgPool, username:String)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(User,
        "UPDATE users SET is_deleted=$1 WHERE user_name = $2 ",true, username.clone() )
            .execute(pool).await?;
        
        Ok(())
    }

    pub async fn user_email_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)",
        email
        )
            .fetch_one(pool)
            .await?;

        Ok(exists.unwrap_or(false))
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

