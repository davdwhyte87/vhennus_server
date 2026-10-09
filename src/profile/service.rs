use std::error::Error;

use sqlx::PgPool;

use crate::profile::models::{MiniProfile, Profile, ProfileWithFriends};
use crate::profile::repository::ProfileRepo;

pub const PROFILE_COLLECTION:&str = "Profile";

pub struct  ProfileService{

}

impl ProfileService {

    pub async fn get_profile(pool:&PgPool, xuser_name:String)->Result<Profile, Box<dyn Error>>{
        ProfileRepo::get_profile(pool, xuser_name).await
    }

    pub async fn get_profile_with_friend(pool:&PgPool, xuser_name:String)->Result<ProfileWithFriends, Box<dyn Error>>{
        ProfileRepo::get_profile_with_friend(pool, xuser_name).await
    }

    pub async fn update_profile(
        pool: &PgPool,
        profile:Profile
    ) -> Result<MiniProfile, Box<dyn Error>> {
        ProfileRepo::update_profile(pool, profile).await
    }

    pub async fn search_users(pool: &PgPool, search_term: String) -> Result<Vec<MiniProfile>, Box<dyn std::error::Error>> {
        ProfileRepo::search_users(pool, search_term).await
    }


    pub async fn user_exists(pool: &PgPool, user_name: &str) -> Result<bool, sqlx::Error> {
        ProfileRepo::user_exists(pool, user_name).await
    }

    pub async fn profile_exists(pool: &PgPool, user_name: &str) -> Result<bool, sqlx::Error> {
        ProfileRepo::profile_exists(pool, user_name).await
    }
    
    
    pub async fn friend_suggestion(pool:&PgPool) ->Result<Vec<MiniProfile>, Box<dyn Error>> {
        ProfileRepo::friend_suggestion(pool).await
    }

    pub async fn get_all(pool:&PgPool)->Result<Vec<Profile>, Box<dyn Error>>{
        ProfileRepo::get_all(pool).await
    }

    pub async fn set_membership(pool:&PgPool, user_name:String, is_member:bool)->Result<(), Box<dyn Error>>{
        ProfileRepo::set_membership(pool, user_name, is_member).await
    }

    pub async fn set_phone_number(pool:&PgPool, user_name:String, phone:String)->Result<(), Box<dyn Error>>{
        ProfileRepo::set_phone_number(pool, user_name, phone).await
    }

    pub async fn update_contact_info(
        pool:&PgPool,
        user_name:String,
        phone:Option<String>,
        country_of_origin:Option<String>,
        state_of_origin:Option<String>,
        date_of_birth:Option<chrono::NaiveDate>,
        current_country:Option<String>,
    )->Result<(), Box<dyn Error>>{
        ProfileRepo::update_contact_info(
            pool, user_name, phone, country_of_origin, state_of_origin, date_of_birth,
            current_country,
        ).await
    }
}