
use std::error::Error;

use futures::StreamExt;
use mongodb::{bson::{doc, from_document}, Database};
use sqlx::PgPool;
use crate::controllers::service_errors::ServiceError;
use crate::models::{chat::Chat, chat_pair::ChatPair};
use crate::models::chat_pair::ChatPairView;
use crate::utils::general::get_time_naive;

pub const CHAT_PAIR_COLLECTION:&str = "ChatPair";

pub struct  ChatPairService{

}

impl ChatPairService {
    pub async fn create_chat_pair(pool:&PgPool, chat:&ChatPair)->Result<ChatPair, Box<dyn Error>>{
        let res_chat = chat.clone();
       let res = sqlx::query_as!(ChatPair,
           "INSERT INTO chat_pairs (id, user1,user2,created_at,updated_at,last_message)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
           chat.id, chat.user1, chat.user2, chat.created_at, chat.updated_at, chat.last_message
       ) 
           .execute(pool).await?;
        if res.rows_affected() == 0 {
            return Err(Box::from("Could not create pair"))
        }
        Ok(res_chat)
    }

    pub async fn update_chat_pair(pool:&PgPool, id:String, last_message:String)->Result<(), ServiceError>{
        let date = get_time_naive();
        let res = sqlx::query_as!(ChatPair,
           "UPDATE chat_pairs 
            SET 
                last_message =COALESCE($1, last_message),
                updated_at = COALESCE($2, updated_at)
            WHERE id = $3
            ",
           last_message, date,id
       )
            .execute(pool).await?;
        if res.rows_affected() == 0 {
            return Err(ServiceError::NoUpdatedRow)
        }
        Ok(())
    }

    pub async  fn find_chat_pair(pool:&PgPool, xuser1:String, xuser2:String)->Result<ChatPairView, Box<dyn Error>>{
        let chat_pair = sqlx::query_as!(ChatPairView, "
             SELECT cp.id, cp.user1, cp.user2, cp.last_message, cp.all_read, cp.created_at, 
               cp.updated_at, p1.image AS user1_image, p2.image AS user2_image
            FROM chat_pairs cp 
            JOIN profiles p1 ON p1.user_name = cp.user1
            JOIN profiles p2 ON p2.user_name = cp.user2   
            WHERE user1 = $1 AND user2 = $2 OR user1 = $2 AND user2 =$1
            " , xuser1, xuser2)
            .fetch_one(pool).await?;
        Ok(chat_pair)
    }


    pub async  fn find_chat_pair_by_id(pool:&PgPool, id:String)->Result<ChatPair, Box<dyn Error>>{
        let chat_pair = sqlx::query_as!(ChatPair, "
            SELECT * FROM chat_pairs WHERE id = $1 " , id)
            .fetch_one(pool).await?;
        Ok(chat_pair)
    }

    pub async fn get_all_my_chat_pairs(pool:&PgPool, user_name:String)->Result<Vec<ChatPairView>, Box<dyn Error>>{
        let chat_pairs = sqlx::query_as!(ChatPairView, "
            SELECT 
               cp.id, cp.user1, cp.user2, cp.last_message, cp.all_read, cp.created_at, 
               cp.updated_at, p1.image AS user1_image, p2.image AS user2_image
            FROM chat_pairs cp 
            JOIN profiles p1 ON p1.user_name = cp.user1
            JOIN profiles p2 ON p2.user_name = cp.user2
            WHERE user1 = $1 OR user2 =$1
            " ,user_name)
            .fetch_all(pool).await?;
        Ok(chat_pairs)
    }
}