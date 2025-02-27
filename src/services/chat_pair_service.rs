use diesel::{BoolExpressionMethods, ExpressionMethods};
use diesel::QueryDsl;
use std::error::Error;
use diesel::RunQueryDsl;
use futures::StreamExt;
use mongodb::{bson::{doc, from_document}, Database};
use crate::DbPool;
use crate::models::{chat::Chat, chat_pair::ChatPair};
use crate::schema::chat_pairs::dsl::chat_pairs;
use crate::schema::chat_pairs::{id, user1, user2};
use crate::schema::chats::dsl::chats;
use crate::schema::chats::pair_id;

pub const CHAT_PAIR_COLLECTION:&str = "ChatPair";

pub struct  ChatPairService{

}

impl ChatPairService {
    // pub async fn create_chat_pair(db:&Database, chat:&ChatPair)->Result<ChatPair, Box<dyn Error>>{
    //     let collection = db.collection::<ChatPair>(CHAT_PAIR_COLLECTION);
    // 
    //     // check if pair exists 
    // 
    //     let filter = doc! {
    //         "users_ids": {"$all":chat.users_ids.clone()}
    //     };
    //     match collection.find_one(filter).await {
    //         Ok(data)=>{
    //             match data{
    //                 Some(data)=>{
    //                     return Ok(data)
    //                 }, 
    //                 None=>{
    //                     // the chat pair does not exist 
    //                     match collection.insert_one(chat).await {
    //                         Ok(data)=>{},
    //                         Err(err)=>{
    //                             log::error!(" error creating  chat pair  {}", err.to_string());
    //                             return Err(err.into()) 
    //                         }
    //                     }
    //                     return Ok(chat.clone())
    //                 }
    //             }
    //         },
    //         Err(err)=>{
    //             log::error!(" error finding  chat pair  {}", err.to_string());
    //             return Err(err.into()) 
    //         }
    //     }
    //  
    //     
    // }

    pub async  fn find_chat_pair(pool:&DbPool, xuser1:String, xuser2:String)->Result<ChatPair, Box<dyn Error>>{
        let xpool = pool.clone();
        let result = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            use crate::schema::chats as schats;
            let chats_data =match chat_pairs.filter(user1.eq(&xuser1).and(user2.eq(&xuser2).or(user1.eq(&xuser2).and(user2.eq(&xuser1)))))
                .first::<ChatPair>(&mut conn){
                Ok(data)=>{data},
                Err(err)=>{
                    return Err(Box::new(err));
                }
            };
            return Ok(chats_data);
        }).await??;
        Ok(result)
    }


    pub async  fn find_chat_pair_by_id(pool:&DbPool, user_id:String)->Result<ChatPair, Box<dyn Error>>{
        let xpool = pool.clone();
        let result = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            use crate::schema::chats as schats;
            let chats_data =match chat_pairs.filter(id.eq(user_id)).first::<ChatPair>(&mut conn){
                Ok(data)=>{data},
                Err(err)=>{
                    return Err(Box::new(err));
                }
            };
            return Ok(chats_data);
        }).await??;
        Ok(result)
    }

    pub async fn get_all_my_chat_pairs(pool:&DbPool, user_name:String)->Result<Vec<ChatPair>, Box<dyn Error>>{
        let xpool = pool.clone();
        let result = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            use crate::schema::chats as schats;
            let chats_data =match chat_pairs.filter(user1.eq(&user_name).or(user2.eq(&user_name))).load::<ChatPair>(&mut conn){
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