use std::error::Error;
use chrono::NaiveDateTime;
use diesel::RunQueryDsl;
use futures::{future::ok, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document}, Database};

use crate::{models::{fried_request::{FriendRequest, FriendRequestStatus}, profile::Profile}, utils::general::get_current_time_stamp, DbPool};
use crate::schema::friend_requests::dsl::friend_requests;
use super::{mongo_service::MongoService, profile_service::PROFILE_COLLECTION};
use diesel::prelude::*;
use crate::database::db::db::DB;
use crate::schema::friend_requests::{id, requester, status, user_name};
use crate::schema::profiles::dsl::profiles;
use crate::schema::{friend_requests as fr, friends};
use crate::schema::profiles as p;
use crate::services::user_service::UserService;
use diesel::result::Error as DieselError;
use serde_derive::Serialize;
use crate::models::profile::Friend;

pub const FRIEND_REQUEST_COLLECTION:&str = "FriendRequest";

pub struct  FriendRequestService{

}
#[derive(Queryable, Default, Serialize)]
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


    pub async fn get_user_friend_request(pool:&DbPool, other_user_name:String)->Result<Vec<FriendRequestWithProfile>, Box<dyn Error>>{
        let conn = &mut pool.get().expect("Couldn't get DB connection");
        match friend_requests.inner_join(profiles.on(requester.eq(other_user_name.clone()).and(status.eq("PENDING")))).select((
            fr::id, fr::user_name, fr::requester, fr::status, fr::created_at,
            p::bio, p::name, p::image
            )).load::<FriendRequestWithProfile>(conn){
            Ok(data) => {
                return Ok(data);
            }
            Err(err) => {
                log::error!("error getting friend requests .. {}", err);
                return Err(err.into());
            }
        };
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


    pub async fn create_friend_request(pool:&DbPool, request:FriendRequest)->Result<(),Box<dyn Error>>{
        // check if the user exists
        match UserService::get_by_username(pool, request.user_name.clone()).await{
            Ok(user) => {},
            Err(err) => {
                log::error!("error getting  user  .. {}", err);
                return Err(err.into());
            }
        };
        match UserService::get_by_username(pool, request.requester.clone()).await{
            Ok(user) => {},
            Err(err) => {
                log::error!("error getting  user  .. {}", err);
            }
        };
        let xpool = pool.clone();
        let mut conn = xpool.get().expect("Couldn't get DB connection");

        let result = actix_web::web::block(move||{
            diesel::insert_into(friend_requests).values(&request).execute(&mut conn)
        }).await;
        match result {
            Ok(data)=>{},
            Err(err)=>{
                log::error!("error saving friend requests .. {}", err);
                return Err(err.into());
            }
        }
        // create friend request
        Ok(())
    }
    
    pub async fn reject_friend_request(pool:&DbPool, frid:String, owner_username:String)->Result<(),Box<dyn Error>>{
        let xpool = pool.clone();
        let res =actix_web::web::block(move||{
            let conn = &mut xpool.get().expect("Couldn't get DB connection");
            let upd = match diesel::update(friend_requests.filter(fr::id.eq(frid)))
                .set(status.eq("REJECTED"))
                .execute(conn){
                Ok(_) => {},
                Err(err) => {
                    log::error!("error updating friend requests .. {}", err);
                    return Err(err.to_string());
                }
            };
            Ok(())
        }).await;
        match res {
            Ok(data)=>{
                return Ok(());
            }
            Err(err)=>{
                log::error!("error rejecting friend request ... {}", err);
                return Err(err.into());
            }
        }
    }


    pub async fn delete_friend_request(pool:&DbPool, request_id:String)->Result<(), Box<dyn Error>>{
        let conn = &mut pool.get().expect("Couldn't get DB connection");

        let deleted = match diesel::delete(friend_requests.filter(id.eq(request_id))).execute(conn){
            Ok(data)=>{
                if data == 0 {
                    return Err(Box::new(diesel::result::Error::NotFound));
                }
            },
            Err(err)=>{
                log::error!("error deleting friend request .. {}", err);
                return Err(err.into())
            }
        };
        Ok(())
    }

    
    pub async fn accept_friend_request(pool:&DbPool, request_id:String, owner_user_name:String)->Result<(),Box<dyn Error>> {
       let xpool = pool.clone();
        let res = actix_web::web::block(move||{
            let mut conn = &mut xpool.get().expect("Couldn't get DB connection");
            let request = match friend_requests.filter(id.eq(request_id.clone())).first::<FriendRequest>(conn){
                Ok(data)=>{data},
                Err(err)=>{
                    log::error!("error getting friend request .. {}", err);
                    return Err(err.to_string());
                }
            };
            
            // check if the req owner is the one who owns the friend request
            if request.user_name.clone() != owner_user_name{
                return Err("UNAUTHORIZED".parse().unwrap());
            }

            use crate::schema::friend_requests;
            // update fr
            let result: Result<(), DieselError> = conn
                .transaction(|conn| {
                    match diesel::update(friend_requests.filter(id.eq(request_id)))
                        .set(friend_requests::status.eq("ACCEPTED"))
                        .execute(conn){
                        Ok(_) => {},
                        Err(err) => {
                            log::error!("error updating friend request .. {}", err);
                            return Err(err.into());
                        }
                    };

                    // create friend 
                    let friend = Friend{
                        id: 0,
                        user_username: request.user_name.clone(),
                        friend_username: request.requester.clone()
                    };
                    match diesel::insert_into(friends::table).values(&friend)
                        .execute(conn){
                        Ok(_) => {},
                        Err(err) => {
                            log::error!("error creating friend .. {}", err);
                            return Err(err.into());
                        }
                    };
                    return Ok(())
                });

            return Ok(())
        }).await;
        
        
        match res{
            Ok(_)=>{},
            Err(err)=>{
                log::error!("error accepting friend request .. {}", err);
                return Err(err.into());
            }
        }
        Ok(())
    }

}