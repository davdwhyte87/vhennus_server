
use std::error::Error;

use futures::StreamExt;
use mongodb::{bson::{doc, from_document}, Database};
use sqlx::PgPool;
use crate::models::{chat::Chat, chat_pair::ChatPair};


pub const CHAT_PAIR_COLLECTION:&str = "ChatPair";

pub struct  ChatPairService{

}

impl ChatPairService {
    pub async fn create_chat_pair(pool:&PgPool, chat:&ChatPair)->Result<ChatPair, Box<dyn Error>>{
        let res_chat = chat.clone();
       let res = sqlx::query_as!(ChatPair,
           "INSERT INTO chat_pairs (id, user1,user2,created_at,updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ",
           chat.id, chat.user1, chat.user2, chat.created_at, chat.updated_at
       ) 
           .execute(pool).await?;
        if res.rows_affected() == 0 {
            return Err(Box::from("Could not create pair"))
        }
        Ok(res_chat)
    }

    pub async  fn find_chat_pair(pool:&PgPool, xuser1:String, xuser2:String)->Result<ChatPair, Box<dyn Error>>{
        let chat_pair = sqlx::query_as!(ChatPair, "
            SELECT * FROM chat_pairs WHERE user1 = $1 AND user2 = $2 OR user1 = $2 AND user2 =$1
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

    pub async fn get_all_my_chat_pairs(pool:&PgPool, user_name:String)->Result<Vec<ChatPair>, Box<dyn Error>>{
        let chat_pairs = sqlx::query_as!(ChatPair, "
            SELECT * FROM chat_pairs WHERE user1 = $1 OR user2 =$1
            " ,user_name)
            .fetch_all(pool).await?;
        Ok(chat_pairs)
    }
}