

use std::{error::Error, vec};
use chrono::NaiveDateTime;

use futures::{future::OkInto, StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document, Regex}, results::{InsertOneResult, UpdateResult}, Database};
use r2d2_mongodb::mongodb::coll;
use serde_derive::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::models::profile::Profile;
use crate::models::user::User;

pub const PROFILE_COLLECTION:&str = "Profile";

pub struct  ProfileService{

}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct MiniProfile{
    pub user_name:String,
    pub image: Option<String>,
    pub bio:Option<String>,
    pub name: Option<String>
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProfileWithFriends{
    profile: Profile,
    friends: Vec<MiniProfile>
}
impl ProfileService {

    pub async fn get_profile(pool:&PgPool, xuser_name:String)->Result<Profile, Box<dyn Error>>{
        let profile = sqlx::query_as!(Profile,
            "SELECT * FROM profiles WHERE user_name = $1", xuser_name)
            .fetch_one(pool)
            .await;
        let profile = match profile{
            Ok(profile)=>profile,
            Err(err)=>{
                return Err(Box::new(err));
            }
        };
        return Ok(profile);
    }
    pub async fn get_profile_with_friend(pool:&PgPool, xuser_name:String)->Result<ProfileWithFriends, Box<dyn Error>>{

        let profile = sqlx::query_as!(Profile,
            "SELECT * FROM profiles WHERE user_name = $1", xuser_name)
            .fetch_one(pool)
            .await;
        let profile = match profile{
            Ok(profile)=>profile,
            Err(err)=>{
                return Err(Box::new(err));
            }
        };
        let friends = sqlx::query_as!(MiniProfile,
        "SELECT p.user_name, p.image, p.bio, p.name FROM profiles p
         JOIN friends f ON p.user_name = f.friend_username OR p.user_name = f.user_username
         WHERE (f.user_username = $1 OR f.friend_username = $1) 
           AND p.user_name <> $1
        ", xuser_name).fetch_all(pool).await?;
        
        let result = ProfileWithFriends{profile, friends};
        return Ok(result);
    }

    pub async fn update_profile(
        pool: &PgPool,
        profile:Profile
    ) -> Result<MiniProfile, Box<dyn Error>> {
        
        let updated_profile = sqlx::query_as!(
        MiniProfile,
        "UPDATE profiles
         SET 
             name = COALESCE($2, name),
             bio = COALESCE($3, bio),
             image = COALESCE($4, image),
             app_f_token = COALESCE($5, app_f_token),
             wallets = COALESCE($6, wallets)
         WHERE user_name = $1
         RETURNING user_name, name, bio, image",
            profile.user_name,
            profile.name,
            profile.bio,
            profile.image,
            profile.app_f_token,
            profile.wallets
        )
            .fetch_one(pool)
            .await?;
        return Ok(updated_profile)
    }

    pub async fn search_users(pool: &PgPool, search_term: String) -> Result<Vec<MiniProfile>, Box<dyn std::error::Error>> {
        let query_param = format!("%{}%", search_term); // Add wildcard for partial match

        let profiles = sqlx::query_as!(
        MiniProfile,
            "SELECT user_name, name, bio, image 
             FROM profiles
             WHERE user_name ILIKE $1 
                OR name ILIKE $1 
                OR bio ILIKE $1",
            query_param
        )
            .fetch_all(pool)
            .await?;

        Ok(profiles)
    }


    pub async fn user_exists(pool: &PgPool, user_name: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE user_name = $1)",
        user_name
        )
            .fetch_one(pool)
            .await?;

        Ok(exists.unwrap_or(false))
    }

    pub async fn profile_exists(pool: &PgPool, user_name: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM profiles WHERE user_name = $1)",
        user_name
    )
            .fetch_one(pool)
            .await?;

        Ok(exists.unwrap_or(false))
    }
    
    
    pub async fn friend_suggestion(pool:&PgPool) ->Result<Vec<MiniProfile>, Box<dyn Error>> {
        let suggestions = sqlx::query_as!(MiniProfile, "
            SELECT user_name, name, bio, image 
            FROM profiles 
            WHERE name IS NOT NULL AND bio IS NOT NULL 
            ORDER BY RANDOM() 
            LIMIT 10;
        ").fetch_all(pool).await?;
        
        Ok(suggestions)
    }
    pub async fn get_all(pool:&PgPool)->Result<Vec<Profile>, Box<dyn Error>>{
        let profiles =match  sqlx::query_as!(Profile, 
        "SELECT * FROM profiles")
            .fetch_all(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(profiles);
    }
}

