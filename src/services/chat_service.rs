use crate::models::chat::Chat;
use crate::models::chat_pair::ChatPair;
use std::error::Error;
use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, Database};
use sqlx::PgPool;
use crate::services::profile_service::ProfileService;
use uuid::Uuid;
use crate::services::chat_pair_service::ChatPairService;
use crate::utils::general::get_time_naive;

pub const CHAT_COLLECTION:&str = "Chat";

pub struct  ChatService{

}

impl ChatService {
    pub async fn create_chat(pool:&PgPool, chat:Chat)->Result<Chat, Box<dyn Error>>{
        let xpool = pool.clone();
        // check that the users exist
        ProfileService::user_exists(pool, &*chat.sender.clone()).await?;
        ProfileService::user_exists(pool, &*chat.receiver.clone()).await?;
        
        // get chat pair 
        let chat_pair = sqlx::query_as!(ChatPair, "
            SELECT * FROM chat_pairs WHERE user1 = $1 AND user2 = $2 OR user1 = $2 AND user2 =$1
            " , chat.sender.clone(), chat.receiver.clone())
            .fetch_optional(pool).await?;
        let mut chat_pair_id = "".to_string();
        if chat_pair.is_some() {
            chat_pair_id = chat_pair.unwrap().id;
        }else{
            chat_pair_id = Uuid::new_v4().to_string();
            let chat_pair = ChatPair{
                id: chat_pair_id.clone(),
                user1: chat.sender.clone(),
                user2: chat.receiver.clone(),
                last_message: None,
                all_read: false,
                created_at: get_time_naive(),
                updated_at: get_time_naive(),
            };
            ChatPairService::create_chat_pair(pool, &chat_pair).await?;
        }
        
        // construct new chat 
        let mut chat = chat;
        chat.id =  Uuid::new_v4().to_string();
        chat.pair_id = chat_pair_id;
        chat.created_at = get_time_naive();
        chat.updated_at = get_time_naive();
        
        let result  = chat.clone();
        
        //create chat 
        let res = sqlx::query_as!(Chat, "
            INSERT INTO chats (id,sender,receiver,message, image,created_at,updated_at, pair_id)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ", chat.id,chat.sender, chat.receiver,chat.message,chat.image
            ,chat.created_at,chat.updated_at,chat.pair_id)
            .execute(pool).await?;
        
        Ok(result)
    }


    // pub async fn get_user_chats(db:&Database, user_name:String)->Result<Vec<Chat>, Box<dyn Error>>{
    //     let collection = db.collection::<Chat>(CHAT_COLLECTION);
    // 
    //     let filter = doc! {
    //         "$or":[
    //             {"sender":user_name.clone()},
    //             {"receiver": user_name.clone()}
    //         ]
    //     };
    //     let mut chats_res =match collection.find(filter).await{
    //         Ok(data)=>{data},
    //         Err(err)=>{
    //             log::error!(" error finding  chat  {}", err.to_string());
    //             return Err(err.into())
    //         }
    //     };
    // 
    //     let mut chats:Vec<Chat> = vec![];
    // 
    //     if let Some(data) = chats_res.next().await{
    //         match data{
    //             Ok(x)=>{
    //                 chats.push(x);
    //             },
    //             Err(err)=>{
    //                 log::error!(" error getting  chat  {}", err.to_string());
    //                 return Err(err.into())   
    //             }
    //         }
    //     }else{
    //         return Err(Box::from("Error reaching chat"));
    //     }
    //     Ok(chats)
    // }

    pub async fn get_chats_by_pair_id(pool:&PgPool, id:String)->Result<Vec<Chat>, Box<dyn Error>>{
        let chats = sqlx::query_as!(Chat, "
            SELECT * FROM chats WHERE pair_id = $1", id)
            .fetch_all(pool).await?;
        Ok(chats)
    }
}