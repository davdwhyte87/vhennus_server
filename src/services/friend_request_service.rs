
use crate::models::fried_request::FriendRequest;
use std::error::Error;
use chrono::NaiveDateTime;
use chrono::Weekday::Fri;
use futures::{future::ok, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document}, Database};

use crate::database::db::db::DB;

use crate::services::user_service::UserService;

use serde_derive::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use crate::controllers::service_errors::ServiceError;
use crate::models::profile::Friend;
use crate::services::profile_service::ProfileService;

pub const FRIEND_REQUEST_COLLECTION:&str = "FriendRequest";

pub struct  FriendRequestService{

}
#[derive( Default, Serialize)]
pub struct FriendRequestWithProfile{
    pub id: String,
    pub user_name: String,
    pub requester:String,
    pub status:String,
    pub created_at: NaiveDateTime,
    pub bio:Option<String>,
    pub name:Option<String>,
    pub image:Option<String>,
}



impl FriendRequestService {


    pub async fn get_user_friend_request(pool:&PgPool, other_user_name:String)->Result<Vec<FriendRequestWithProfile>, Box<dyn Error>>{
        let chat_pair = sqlx::query_as!(FriendRequestWithProfile, "
            SELECT fr.id,fr.user_name,fr.requester, fr.status,fr.created_at,
            p.bio,p.name,p.image    
            FROM friend_requests fr
            INNER JOIN profiles p ON p.user_name = fr.requester
            WHERE fr.user_name = $1 AND fr.status = $2
            " , other_user_name, "PENDING")
            .fetch_all(pool).await?;
        Ok(chat_pair)
    }

    pub async fn get_single_friend_request(pool:&PgPool, id:String)->Result<FriendRequest, Box<dyn Error>>{
        let fr = sqlx::query_as!(FriendRequest, "
            SELECT *    
            FROM friend_requests 
            WHERE id = $1
            " , id)
            .fetch_one(pool).await?;
        Ok(fr)
    }

    pub async fn get_single_friend_request_by_users(pool:&PgPool, requester:String, user:String)->Result<Option<FriendRequest>, ServiceError>{
        let fr = sqlx::query_as!(FriendRequest, "
            SELECT *    
            FROM friend_requests 
            WHERE requester = $1 AND user_name=$2
            " , requester, user)
            .fetch_optional(pool).await?;
        Ok(fr)
    }
    
    // pub async fn get_single_friend_request(db:&Database, id:String)->Result<Option<FriendRequest>, Box<dyn Error>>{
    //     // Get a handle to a collection in the database.
    //     let collection = db.collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);
    //
    //     let mut res = match collection.find_one(doc! {"id": id}).await{
    //         Ok(data)=>{data},
    //         Err(err)=>{
    //             log::error!(" error fetching friend request data  {}", err.to_string());
    //             return Err(err.into())
    //         }
    //     };
    //
    //
    //     Ok(res)
    // }


    pub async fn create_friend_request(pool:&PgPool, request:FriendRequest)->Result<(), ServiceError>{
        // check if the user exists
        ProfileService::user_exists(pool, &*request.requester.clone()).await
            .map_err(|_| ServiceError::UserNotFound)?;
        ProfileService::user_exists(pool, &*request.user_name.clone()).await
            .map_err(|_| ServiceError::UserNotFound)?;
        //check if it exists 
        let frr = Self::get_single_friend_request_by_users(pool, request.requester.clone(), request.user_name.clone()).await?;
        if frr.is_some(){
            return Err(ServiceError::FriendRequestExists);
        }
        // create friend request
        let res = sqlx::query_as!(FriendRequest, "
          INSERT INTO friend_requests (id,user_name,requester, status, created_at, updated_at)
          VALUES ($1,$2,$3,$4,$5,$6)",
            request.id,request.user_name,request.requester,request.status,request.created_at,request.updated_at
        ).execute(pool).await?;
        Ok(())
    }
    
    pub async fn reject_friend_request(pool:&PgPool, frid:String, owner_username:String)->Result<(),Box<dyn Error>>{
        let res = sqlx::query_as!(FriendRequest, 
            "UPDATE friend_requests SET status =COALESCE($3,status) WHERE id = $1 AND user_name = $2",
            frid, owner_username,"REJECTED"
        ).execute(pool).await?;
     return Ok(())
    }

    pub async fn reject_friend_request2(pool:&PgPool, frid:String, owner_username:String)->Result<(),Box<dyn Error>>{
        let res = sqlx::query_as!(FriendRequest, 
            "DELETE FROM friend_requests  WHERE id = $1 AND user_name = $2",
            frid, owner_username
        ).execute(pool).await?;
        return Ok(())
    }


    // pub async fn delete_friend_request(pool:&DbPool, request_id:String)->Result<(), Box<dyn Error>>{
    //     let conn = &mut pool.get().expect("Couldn't get DB connection");
    // 
    //     let deleted = match diesel::delete(friend_requests.filter(id.eq(request_id))).execute(conn){
    //         Ok(data)=>{
    //             if data == 0 {
    //                 return Err(Box::new(diesel::result::Error::NotFound));
    //             }
    //         },
    //         Err(err)=>{
    //             log::error!("error deleting friend request .. {}", err);
    //             return Err(err.into())
    //         }
    //     };
    //     Ok(())
    // }

    
    pub async fn accept_friend_request(pool:&PgPool, request_id:String, owner_user_name:String)->Result<(),Box<dyn Error>> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
        // get friend request 
        let fr = Self::get_single_friend_request(pool, request_id.clone()).await?;
        
        // update friend request 
        let res = sqlx::query_as!(FriendRequest, 
            "UPDATE friend_requests SET status =COALESCE($3,status) WHERE id = $1 AND user_name = $2",
            request_id, owner_user_name,"ACCEPTED"
        ).execute(&mut *tx).await?;
     
        if res.rows_affected() == 0 {
            tx.rollback().await?;
            return  return Err("Failed to update friend request".into());
        }
        
        let friend = Friend{
            id: 0,
            user_username: "".to_string(),
            friend_username: "".to_string(),
        };
        let result = sqlx::query_as!(Friend, "
            INSERT INTO friends (user_username,friend_username)
            VALUES ($1,$2)
            ",owner_user_name, fr.requester )
            .execute(&mut *tx).await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err("Failed to create friend".into());
        }
        tx.commit().await?;
        return Ok(())
    }

}