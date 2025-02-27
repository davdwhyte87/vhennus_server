use std::error::Error;
use diesel::{BoolExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, Database};

use crate::{models::{chat::Chat, chat_pair::ChatPair, user::{self, User}}, utils::general::get_current_time_stamp, DbPool};
use crate::schema::chat_pairs::dsl::chat_pairs;
use crate::schema::chat_pairs::{user1, user2};
use crate::schema::users::dsl::users;
use crate::services::profile_service::ProfileService;
use super::{chat_pair_service::CHAT_PAIR_COLLECTION, user_service::USER_COLLECTION};
use diesel::prelude::*;
use uuid::Uuid;
use crate::schema::chats::dsl::chats;
use crate::schema::chats::pair_id;

pub const CHAT_COLLECTION:&str = "Chat";

pub struct  ChatService{

}

impl ChatService {
    pub async fn create_chat(pool:&DbPool, chat:Chat)->Result<Chat, Box<dyn Error>>{
        let xpool = pool.clone();
        // check that the users exist
        ProfileService::user_exists(pool, chat.sender.clone()).await?;
        ProfileService::user_exists(pool, chat.receiver.clone()).await?;
        
        let mut chat = chat.clone();
        // get the chat pair
        let result = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            let mut conn2 = xpool.get().expect("Couldn't get DB connection");
            let mut conn3 = xpool.get().expect("Couldn't get DB connection");
            let chat_pairs_data = match chat_pairs
                .filter(
                    (user1.eq(&chat.sender).and(user2.eq(&chat.receiver)))
                        .or(user1.eq(&chat.receiver).and(user2.eq(&chat.sender)))
                )
                .first::<ChatPair>(&mut conn)
                .optional(){
                Ok(data)=>{data},
                Err(err)=>{
                    return Err(Box::new(err));
                }
            };
            let mut chat_pair_id:String;
            if chat_pairs_data.is_none() {
                // create chat pair that means its a new chatu
                use crate::schema::chat_pairs as schat_pairs;
                let new_chat_pair = ChatPair{
                    id: Uuid::new_v4().to_string(),
                    user1: chat.sender.clone(),
                    user2: chat.receiver.clone(),
                    last_message: Some(chat.message.clone()),
                    all_read: false,
                    created_at: Default::default(),
                    updated_at: Default::default(),
                };
                match diesel::insert_into(schat_pairs::table).values(&new_chat_pair)
                    .execute(&mut conn2){
                    Ok(data)=>{},
                    Err(err)=>{
                        return Err(Box::new(err));
                    }
                }
                chat_pair_id=new_chat_pair.id;
            }else{
                chat_pair_id = chat_pairs_data.unwrap().id;
                
            }
            
            // create new chat
            chat.pair_id = chat_pair_id.clone();
            chat.id = Uuid::new_v4().to_string();
            
            use crate::schema::chats as schats;
            
            match diesel::insert_into(schats::table).values(&chat)
                .execute(&mut conn3){
                Ok(data)=>{},
                Err(err)=>{
                    return Err(Box::new(err));
                }
            };
            return Ok(chat);

        }).await??;

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

    pub async fn get_chats_by_pair_id(pool:&DbPool, id:String)->Result<Vec<Chat>, Box<dyn Error>>{
        let xpool = pool.clone();
        
        let result = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            use crate::schema::chats as schats;
            let chats_data =match chats.filter(pair_id.eq(id)).load::<Chat>(&mut conn){
                Ok(data)=>{data},
                Err(err)=>{
                    return Err(Box::new(err));
                }
            };
            return Ok(chats_data);
        }).await??;
        
        Ok(result)
    }
}