use std::error::Error;

use futures::StreamExt;
use mongodb::{bson::{doc, from_document}, Database};

use crate::models::{chat::Chat, chat_pair::ChatPair};




pub const CHAT_PAIR_COLLECTION:&str = "ChatPair";

pub struct  ChatPairService{

}

impl ChatPairService {
    pub async fn create_chat_pair(db:&Database, chat:&ChatPair)->Result<ChatPair, Box<dyn Error>>{
        let collection = db.collection::<ChatPair>(CHAT_PAIR_COLLECTION);

        // check if pair exists 

        let filter = doc! {
            "users_ids": {"$all":chat.users_ids.clone()}
        };
        match collection.find_one(filter).await {
            Ok(data)=>{
                match data{
                    Some(data)=>{
                        return Ok(data)
                    }, 
                    None=>{
                        // the chat pair does not exist 
                        match collection.insert_one(chat).await {
                            Ok(data)=>{},
                            Err(err)=>{
                                log::error!(" error creating  chat pair  {}", err.to_string());
                                return Err(err.into()) 
                            }
                        }
                        return Ok(chat.clone())
                    }
                }
            },
            Err(err)=>{
                log::error!(" error finding  chat pair  {}", err.to_string());
                return Err(err.into()) 
            }
        }
     
        
    }


    pub async fn get_chat_pair_by_id(db:&Database, id:String)->Result<ChatPair, Box<dyn Error>>{
        let collection = db.collection::<ChatPair>(CHAT_PAIR_COLLECTION);


        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "Profile",
                  "localField": "users_ids",
                  "foreignField": "user_name",
                  "as": "users"
               }
        };
        let filter = doc! {"$match":doc! {"id":id}}; 
        let mut results = collection.aggregate(vec![filter,lookup_2]).await?;
        let mut chat_pair: ChatPair = ChatPair::default();
        while let Some(data) = results.next().await{
            match data {
                Ok(data)=>{
                    chat_pair = from_document(data)?;

                },
                Err(err)=>{
                    log::error!(" error finding  chat pair  {}", err.to_string());
                    return Err(err.into())    
                }
            }
        }
        // let chat_pair = match collection.find_one(doc! {"id":id}).await{
        //     Ok(data)=>{
        //         data
        //     },
        //     Err(err)=>{
        //         log::error!(" error finding  chat pair  {}", err.to_string());
        //         return Err(err.into())  
        //     }
        // };
        Ok(chat_pair)

    }


    pub async fn get_all_my_chat_pairs(db:&Database, user_name:String)->Result<Vec<ChatPair>, Box<dyn Error>>{
        let collection = db.collection::<ChatPair>(CHAT_PAIR_COLLECTION);
        
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "Profile",
                  "localField": "users_ids",
                  "foreignField": "user_name",
                  "as": "users"
               }
        };
        let filter = doc! {"$match": 
        doc! {
            "users_ids": user_name
            }
        };
        let mut results = collection.aggregate(vec![filter,lookup_2]).await?;
        let mut chat_pairs: Vec<ChatPair> = vec![];
        while let Some(data) = results.next().await{
            match data {
                Ok(data)=>{
                    let chat_pair:ChatPair = from_document(data)?;
                    chat_pairs.push(chat_pair);

                },
                Err(err)=>{
                    log::error!(" error finding  chat pair  {}", err.to_string());
                    return Err(err.into())    
                }
            }
        }
    
        Ok(chat_pairs)

    }
}