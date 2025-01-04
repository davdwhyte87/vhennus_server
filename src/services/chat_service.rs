use std::error::Error;

use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, Database};

use crate::{models::{chat::Chat, chat_pair::ChatPair, user::{self, User}}, utils::general::get_current_time_stamp};

use super::{chat_pair_service::CHAT_PAIR_COLLECTION, user_service::USER_COLLECTION};



pub const CHAT_COLLECTION:&str = "Chat";

pub struct  ChatService{

}

impl ChatService {
    pub async fn create_chat(db:&Database, chat:&mut Chat)->Result<Chat, Box<dyn Error>>{
        let collection = db.collection::<Chat>(CHAT_COLLECTION);
        let user_collection = db.collection::<User>(USER_COLLECTION);

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

        // check if  a chat pair exists. if none, create one. 

        let pair_collection = db.collection::<ChatPair>(CHAT_PAIR_COLLECTION);

        // check if pair exists 

        let filter = doc! {
            "users_ids": {"$all":vec![chat.sender.clone(), chat.receiver.clone()]}
        };
        let chat_pair = match pair_collection.find_one(filter).await {
            Ok(data)=>{
                match data{
                    Some(data)=>{
                        data
                    }, 
                    None=>{
                        // the chat pair does not exist 
                        // create new one 
                        let chat_pair = ChatPair{
                            id: uuid::Uuid::new_v4().to_string(),
                            user_name: chat.sender.clone(),
                            users_ids: vec![chat.sender.clone(), chat.receiver.clone()],
                            users: None,
                            last_message: "".to_string(),
                            all_read:true,
                            created_at: get_current_time_stamp(),
                            updated_at:get_current_time_stamp()
                        };
                        match pair_collection.insert_one(chat_pair.clone()).await {
                            Ok(data)=>{
                                chat_pair
                            },
                            Err(err)=>{
                                log::error!(" error creating  chat pair  {}", err.to_string());
                                return Err(err.into()) 
                            }
                        }
                    }
                }
            },
            Err(err)=>{
                log::error!(" error finding  chat pair  {}", err.to_string());
                return Err(err.into()) 
            }
        };

        chat.pair_id = chat_pair.id;
        let res_chat = chat.clone();

        match collection.insert_one(chat).await{
            Ok(data)=>{},
            Err(err)=>{
                log::error!(" error saving  chat  {}", err.to_string());
                return Err(err.into())
            }
        }

        // update the chat pair with last message 
        match pair_collection.update_one(doc! {
            "id":res_chat.pair_id.clone()
        }, doc! {"$set": doc! {"last_message":res_chat.message.clone()}}).await{
            Ok(_)=>{
                // we do not care if it fails... 
            },
            Err(err)=>{
                // log failure
                log::error!(" error updating chat pair  {}", err.to_string());
               
            }
        };
        Ok(res_chat)
    }


    pub async fn get_user_chats(db:&Database, user_name:String)->Result<Vec<Chat>, Box<dyn Error>>{
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
        Ok(chats)
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