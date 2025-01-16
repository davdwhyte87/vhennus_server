use std::error::Error;

use futures::{future::ok, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document}, Database};

use crate::{models::{fried_request::{FriendRequest, FriendRequestStatus}, profile::Profile}, utils::general::get_current_time_stamp};

use super::{mongo_service::MongoService, profile_service::PROFILE_COLLECTION};

pub const FRIEND_REQUEST_COLLECTION:&str = "FriendRequest";

pub struct  FriendRequestService{

}

impl FriendRequestService {



    pub async fn get_user_friend_request(db:&Database, user_name:String)->Result<Vec<FriendRequest>, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);

        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "Profile",
                  "localField": "requester",
                  "foreignField": "user_name",
                  "as": "requester_profile"
               }
        };
        let unwind = doc! {
            "$unwind": {
                "path": "$requester_profile",
                "preserveNullAndEmptyArrays": true
            }
        };

        let filter = doc! {"$match":doc! {"user_name":user_name.clone(), "status":FriendRequestStatus::PENDING.to_string()}}; 
        let mut results = match collection.aggregate(vec![filter,lookup_2, unwind]).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error fetching friend request data  {}", err.to_string());
                return Err(err.into())   
            }
        };

        let mut requests:Vec<FriendRequest> = vec![];


        while let Some(request) = results.try_next().await?{
            // log::error!("document {}", request);
            let x:FriendRequest = match from_document(request){

                Ok(data)=>{
                    data
                },
                Err(err)=>{
                    log::error!(" error converting document to friend request  {}", err.to_string());
                    return Err(err.into())   
                }
            };
            requests.push(x);
        }

        Ok(requests)
    }

    
    pub async fn get_single_friend_request(db:&Database, id:String)->Result<Option<FriendRequest>, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);

        let mut res = match collection.find_one(doc! {"id": id}).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error fetching friend request data  {}", err.to_string());
                return Err(err.into())   
            }
        };
       

        Ok(res)
    }


    pub async fn create_friend_request(db:&Database, request:FriendRequest)->Result<(),Box<dyn Error>>{
        let collection = db.collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);

        // check if there is already a friend request 
        let filter = doc! {
            "user_name": request.user_name.clone(),
            "requester": request.requester.clone()
        };

        let fr = match collection.find_one(filter).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error error geting friend request {}", err.to_string());
                return Err(err.into())  
            }
        };

        match fr {
            Some(_)=>{
                return Err(Box::from("FRIEND_REQUEST_EXISTS")); 
            },
            None=>{

            }
        }
        
        let res = match collection.insert_one(request).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting freind request  {}", err.to_string());
                return Err(err.into())   
            }
        };

        Ok(())
    }


    pub async fn delete_friend_request(db:&Database, id:String)->Result<(), Box<dyn Error>>{
        let collection = db.collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);

        let res = match collection.delete_one(doc! {"id":id}).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error deleting freind request  {}", err.to_string());
                return Err(err.into())   
            }
        };

        Ok(())
    }

    
    pub async fn accept_friend_request(db:&Database, mut request:FriendRequest)->Result<(),Box<dyn Error>>{
        log::debug!("accept FR service starting ...");
        //let collection = db.collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);
        let mut  session = match db.client().start_session().await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!("error creating database session {}", err);
                return Err(err.into());
            }
        };
 
        match session.start_transaction().await{
            Ok(data)=>{},
            Err(err)=>{
                log::error!("error creating session transaction  {}", err);
                return Err(err.into());   
            }
        };

        
        let fr_collection = session.client().database(&MongoService::get_db_name()).collection::<FriendRequest>(FRIEND_REQUEST_COLLECTION);
        let profile_collection = session.client().database(&MongoService::get_db_name()).collection::<Profile>(PROFILE_COLLECTION);
        request.status = FriendRequestStatus::ACCEPTED;
        let update_data = doc! {
            "$set":doc! {
                "status":request.status.to_string(),
            }
        };
        // update the request 
        let res = match fr_collection.update_one(doc! {"id":request.id.clone()},update_data).session(&mut session).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting freind request  {}", err.to_string());
                session.abort_transaction().await;
                return Err(err.into())   
            }
        };

        // get requester profile 
        let mut req_profile = match profile_collection.find_one(doc! {"user_name":request.requester.clone()}).await{
            Ok(data)=>{
                match data{
                    Some(data)=>{data},
                    None=>{
                       // create a profile 
                       let mut re = Profile::default();
                       re.id = uuid::Uuid::new_v4().to_string();
                       re.created_at = get_current_time_stamp();
                       re.updated_at = get_current_time_stamp();
                       re.user_name = request.requester.clone();
                       re.friends = vec![request.user_name.clone()];
                       match profile_collection.insert_one(&re).await {
                        Ok(_)=>{},
                        Err(err)=>{
                            log::error!(" error creating requester profile  {}", err.to_string());
                            session.abort_transaction().await;
                            return Err(err.into())      
                        }
                       }
                       re
                    }
                }
            },
            Err(err)=>{
                log::error!(" error getting user profile  {}", err.to_string());
                session.abort_transaction().await;
                return Err(err.into())   
            }
        };
        //get profile
        let mut profile = match profile_collection.find_one(doc! {"user_name":request.user_name.clone()}).await{
            Ok(data)=>{
                match data{
                    Some(data)=>{data},
                    None=>{
                        log::error!(" error getting user profile, no profile found");
                        session.abort_transaction().await;
                        return Err(Box::from("no user found"))    
                    }
                }
            },
            Err(err)=>{
                log::error!(" error getting user profile  {}", err.to_string());
                session.abort_transaction().await;
                return Err(err.into())   
            }
        };
        //add friend to friend list

        if !profile.friends.contains(&request.requester) {
            profile.friends.push(request.requester);
        }
       
        let profile_update_data = doc! {
            "$set":doc! {
                "friends":profile.friends,
            }
        };

        log::debug!("1 {:?}", req_profile.friends.clone());
        if !req_profile.friends.contains(&request.user_name){
            req_profile.friends.push(request.user_name.clone());
        }

        let req_profile_update_data = doc! {
            "$set":doc! {
                "friends":req_profile.friends.clone(),
            }
        };
        let res = match profile_collection.update_one(doc! {"user_name":profile.user_name.clone()},profile_update_data).session(&mut session).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting freind request  {}", err.to_string());
                session.abort_transaction().await;
                return Err(err.into())   
            }
        };
        // update requester 
        log::debug!("{:?}", req_profile_update_data);
        let res = match profile_collection.update_one(doc! {"user_name":req_profile.user_name.clone()},req_profile_update_data).session(&mut session).await{
            Ok(data)=>{
                log::debug!("{:?}", req_profile.user_name);
            },
            Err(err)=>{
                log::error!(" error inserting freind request  {}", err.to_string());
                session.abort_transaction().await;
                return Err(err.into())   
            }
        };
        session.commit_transaction().await;
        Ok(())
    }



}