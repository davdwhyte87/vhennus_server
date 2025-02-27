
use diesel::{AsChangeset, BoolExpressionMethods, Insertable, JoinOnDsl, OptionalExtension, PgTextExpressionMethods, QueryDsl, Queryable, Selectable};
use std::{error::Error, vec};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, RunQueryDsl};
use futures::{future::OkInto, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document, Regex}, results::{InsertOneResult, UpdateResult}, Database};
use r2d2_mongodb::mongodb::coll;
use serde_derive::{Deserialize, Serialize};
use crate::{models::{buy_order::BuyOrder, post::Post, profile::Profile, sell_order::SellOrder, user::User}, utils::general::get_current_time_stamp, DbPool};
use crate::schema::friends::dsl::friends;
use crate::schema::profiles::dsl::profiles;
use crate::schema::profiles::user_name;
use super::{mongo_service::MongoService, post_service::POST_SERVICE_COLLECTION, user_service::USER_COLLECTION};


pub const PROFILE_COLLECTION:&str = "Profile";

pub struct  ProfileService{

}

#[derive(Serialize,Queryable, Debug, Deserialize, Clone)]
pub struct MiniProfile{
    pub user_name:String,
    pub image: Option<String>,
    pub bio:Option<String>,
    pub name: Option<String>
}
#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable)]
pub struct ProfileWithFriends{
    pub id: String,
    pub user_name:String,
    pub bio: Option<String>,
    pub name:Option<String>,
    pub image:Option<String>,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub app_f_token: Option<String>,
    friends: Vec<MiniProfile>
}
impl ProfileService {

    pub async fn get_profile(pool:&DbPool, xuser_name:String)->Result<Profile, Box<dyn Error>>{
        let xpool = pool.clone();

        let result = actix_web::web::block(move||{
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            let mut  conn2 =  xpool.get().expect("Couldn't get DB connection");
            let profile = match profiles.filter(user_name.eq(&xuser_name)).first::<Profile>(&mut conn){
                Ok(profile)=>{profile},
                Err(err)=>{
                    log::error!("error getting user profile, {}", err);
                    return Err(err);
                }
            };


            return Ok(profile)
        }).await??;

        return Ok(result);
    }
    pub async fn get_profile_with_friend(pool:&DbPool, xuser_name:String)->Result<ProfileWithFriends, Box<dyn Error>>{
        let xpool = pool.clone();

        let result = actix_web::web::block(move||{
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            let mut  conn2 =  xpool.get().expect("Couldn't get DB connection");
            let profile = match profiles.filter(user_name.eq(&xuser_name)).first::<Profile>(&mut conn){
                Ok(profile)=>{profile},
                Err(err)=>{
                    log::error!("error getting user profile, {}", err);
                    return Err(err);
                }
            };
            use crate::schema::friends as fr;
            use crate::schema::profiles as pr;
            let friends_data = match friends.inner_join(profiles.on(fr::friend_username.eq(pr::user_name)))
                .filter(fr::user_username.eq(&xuser_name))
                .select(( user_name,pr::image, pr::bio, pr::name)).load::<MiniProfile>(&mut conn2){
                Ok(data)=>{data},
                Err(err)=>{
                    log::error!("erro getting friends data {}", err);
                    return Err(err);
                }
            };

            return Ok(ProfileWithFriends{
                id: profile.id,
                user_name: profile.user_name,
                bio: profile.bio,
                name: profile.name,
                image: profile.image,
                created_at: profile.created_at,
                updated_at: profile.updated_at,
                app_f_token: profile.app_f_token,
                friends: friends_data,
            })
        }).await??;

        return Ok(result);
    }

    pub async fn update_profile(pool:&DbPool, profile: Profile) -> Result<(), Box<dyn Error>> {

        let xpool = pool.clone();
        let mut conn = &mut pool.get().expect("Couldn't get DB connection");
        // get profile
        let uprofile = match profiles.filter(user_name.eq(profile.user_name.clone()))
            .first::<Profile>(conn){
            Ok(profile) => profile,
            Err(err) => {
                log::error!("error getting profile {}", err);
                return Err(err.into());
            }
        };
        // update profile
        let result = actix_web::web::block(move || {
            let mut conn = &mut xpool.get().expect("Couldn't get DB connection");
            match diesel::update(profiles.filter(user_name.eq(profile.user_name.clone())))
                .set(&profile)
                .execute(conn){
                Ok(_) => {},
                Err(err)=>{
                    log::error!("error updating profile {}", err);
                    return Err(err.to_string());
                }
            }
            Ok(())
        }).await;
        return Ok(())
    }

    pub async fn search_users(pool: &DbPool, search_term: String) -> Result<Vec<MiniProfile>, Box<dyn std::error::Error>> {
        let xpool = pool.clone();
        let search_pattern = format!("%{}%", search_term);

        let users = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");
            use crate::schema::profiles as pr;
            profiles
                .filter(
                    pr::user_name.ilike(&search_pattern)
                        .or(pr::name.ilike(&search_pattern))
                )
                .select((pr::user_name, pr::image, pr::bio, pr::name))
                .load::<MiniProfile>(&mut conn)
        })
            .await??;

        Ok(users)
    }


    pub async fn user_exists(pool: &DbPool, search_username: String) -> Result<bool, Box<dyn std::error::Error>> {
        let xpool = pool.clone();

        let result = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("Couldn't get DB connection");

            profiles
                .filter(user_name.eq(search_username))
                .select(user_name) // Only select username to optimize performance
                .first::<String>(&mut conn)
                .optional() // Returns Some(String) if found, None if not
        }).await??;

        Ok(result.is_some()) // Returns true if user exists, false otherwise
    }
}