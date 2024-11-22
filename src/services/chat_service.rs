use std::error::Error;

use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, Database};

use crate::models::{chat::Chat, user::{self, User}};

use super::user_service::USER_COLLECTION;



pub const CHAT_COLLECTION:&str = "Chat";

pub struct  ChatService{

}

impl ChatService {
    pub async fn create_chat(db:&Database, chat:&Chat)->Result<(), Box<dyn Error>>{
        let collection = db.collection::<Chat>(CHAT_COLLECTION);
        let user_collection = db.collection::<User>(USER_COLLECTION);
        // check if a chat is already created 
        // let filter = doc! {
        //     "$and":[
        //         {"sender":chat.sender.clone()},
        //         {"receiver": chat.receiver.clone()}
        //     ]
        // };
        // let mut chats_res =match collection.find(filter).await{
        //     Ok(data)=>{data},
        //     Err(err)=>{
        //         log::error!(" error finding  chat  {}", err.to_string());
        //         return Err(err.into())
        //     }
        // };
        
        // let mut chats:Vec<Chat> = vec![];
        
        // if let Some(data) = chats_res.next().await{
        //     match data{
        //         Ok(x)=>{
        //             chats.push(x);
        //         },
        //         Err(err)=>{
        //             log::error!(" error getting  chat  {}", err.to_string());
        //             return Err(err.into())   
        //         }
        //     }
        // }

        // log::error!("{:?}", chats);

        // if chats.is_empty() {
        //     log::error!(" No chats");
        // }


        // check if the sender and receiver exist

        match user_collection.find_one(doc! {"user_name":chat.sender.clone()}).await{
            Ok(data)=>{
                match data {
                    Some(_)=>{},
                    None=>{
                        log::error!(" user not found  ");
                        return Err(Box::from("Sender not found"))    
                    }
                }
            },
            Err(err)=>{
                log::error!(" error getting  user  {}", err.to_string());
                return Err(err.into())  
            }
        }

        match user_collection.find_one(doc! {"user_name":chat.receiver.clone()}).await{
            Ok(data)=>{
                match data {
                    Some(_)=>{},
                    None=>{
                        log::error!(" user not found  ");
                        return Err(Box::from("receiver not found"))    
                    }
                }
            },
            Err(err)=>{
                log::error!(" error getting  user  {}", err.to_string());
                return Err(err.into())  
            }
        }
        match collection.insert_one(chat).await{
            Ok(data)=>{},
            Err(err)=>{
                log::error!(" error saving  chat  {}", err.to_string());
                return Err(err.into())
            }
        }
        Ok(())
    }


    pub async fn get_user_chats(db:&Database, user_name:String)->Result<(), Box<dyn Error>>{
        let collection = db.collection::<Chat>(CHAT_COLLECTION);

        let filter = doc! {
            "$or":[
                {"sender":user_name.clone()},
                {"receiver": user_name.clone()}
            ]
        };
        let mut chats_res =match collection.find(filter).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error finding  chat  {}", err.to_string());
                return Err(err.into())
            }
        };

        let mut chats:Vec<Chat> = vec![];

        if let Some(data) = chats_res.next().await{
            match data{
                Ok(x)=>{
                    chats.push(x);
                },
                Err(err)=>{
                    log::error!(" error getting  chat  {}", err.to_string());
                    return Err(err.into())   
                }
            }
        }else{
            return Err(Box::from("Error reaching chat"));
        }
        Ok(())
    }

    pub async fn get_chats_by_pair_id(db:&Database, id:String)->Result<Vec<Chat>, Box<dyn Error>>{
        let collection = db.collection::<Chat>(CHAT_COLLECTION);

        let filter = doc! {"pair_id":id};
        let mut chats_res =match collection.find(filter).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error finding  chat  {}", err.to_string());
                return Err(err.into())
            }
        };

        let mut chats:Vec<Chat> = vec![];

        while let Some(data) = chats_res.try_next().await?{
            chats.push(data);
           
        }
        Ok(chats)
    }
}