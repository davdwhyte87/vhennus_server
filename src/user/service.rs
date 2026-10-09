use std::error::Error;

use rand::Rng;
use sqlx::PgPool;

use crate::profile::service::ProfileService;
use crate::shared::error::ServiceError;
use crate::shared::send_email::EmailService;
use crate::user::models::{FriendRequest, User};
use crate::user::repository::{FriendRequestRepo, UserRepo};

pub const USER_COLLECTION:&str = "User";

pub struct UserService{
    client: mongodb::Client

}

pub struct UserView{
    
}

impl UserService{

    pub async fn create_user(pool:&PgPool, user:User, code:i32)->Result<(), Box<dyn Error>>{
        UserRepo::insert_user_with_profile(pool, user, code).await
    }

    pub async fn get_by_username(pool:&PgPool, username:String)->Result<Option<User>, Box<dyn Error>>{
        UserRepo::get_by_username(pool, username).await
    }

    pub async fn get_all(pool:&PgPool)->Result<Vec<User>, Box<dyn Error>>{
        UserRepo::get_all(pool).await
    }

    pub async fn get_by_email(pool:&PgPool, email:String)->Result<Option<User>, Box<dyn Error>>{
        UserRepo::get_by_email(pool, email).await
    }

    pub async fn confirm_user_email(pool:&PgPool, email:String, code:String)->Result<(), Box<dyn Error>>{
        UserRepo::confirm_user_email(pool, email, code).await
    }
    
    pub async fn update_code(pool:&PgPool, email:String)->Result<(), Box<dyn Error>>{
        let code = rand::thread_rng()
            .gen_range(100_000..1_000_000) ;
        
        UserRepo::update_user_code(pool, email.clone(), code).await?;
        
        // send user email
        let email_service = &EmailService::new();
        match EmailService::send_signup_email2(email_service,email.clone(), code.to_string()).await{
            Ok(email)=>{},
            Err(err)=>{
                log::error!("Failed to send signup email: {}", err);
                return Err(Box::new(err));
            }
        };
        Ok(())
    }

    pub async fn update(pool:&PgPool, user:User)->Result<(), Box<dyn Error>>{
        UserRepo::update(pool, user).await
    }
    
    pub async fn delete_user(pool:&PgPool, username:String)->Result<(), Box<dyn Error>>{
        UserRepo::delete_user(pool, username).await
    }

    pub async fn user_email_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
        UserRepo::user_email_exists(pool, email).await
    }
}

pub struct  FriendRequestService{

}

impl FriendRequestService {

    pub async fn get_user_friend_request(pool:&PgPool, other_user_name:String)->Result<Vec<crate::user::models::FriendRequestWithProfile>, Box<dyn Error>>{
        FriendRequestRepo::get_user_friend_request(pool, other_user_name).await
    }

    pub async fn get_single_friend_request(pool:&PgPool, id:String)->Result<FriendRequest, Box<dyn Error>>{
        FriendRequestRepo::get_single_friend_request(pool, id).await
    }

    pub async fn get_single_friend_request_by_users(pool:&PgPool, requester:String, user:String)->Result<Option<FriendRequest>, ServiceError>{
        FriendRequestRepo::get_single_friend_request_by_users(pool, requester, user).await
    }

    pub async fn create_friend_request(pool:&PgPool, request:FriendRequest)->Result<(), ServiceError>{
        if request.requester == request.user_name {
            return Err(ServiceError::InvalidInput(
                "You cannot send a friend request to yourself".to_string(),
            ));
        }
        // check that both users exist
        ProfileService::user_exists(pool, &request.requester.clone())
            .await
            .map_err(|_| ServiceError::UserNotFound)?;
        ProfileService::user_exists(pool, &request.user_name.clone())
            .await
            .map_err(|_| ServiceError::UserNotFound)?;
        // already friends (either direction) -> no request needed
        if FriendRequestRepo::are_friends(
            pool,
            request.requester.clone(),
            request.user_name.clone(),
        )
        .await?
        {
            return Err(ServiceError::AlreadyFriends);
        }
        // check both directions for existing requests
        let existing = FriendRequestRepo::get_friend_request_between(
            pool,
            request.requester.clone(),
            request.user_name.clone(),
        )
        .await?;
        for row in &existing {
            let same_direction =
                row.requester == request.requester && row.user_name == request.user_name;
            match row.status.as_str() {
                "PENDING" => return Err(ServiceError::FriendRequestExists),
                "ACCEPTED" => return Err(ServiceError::AlreadyFriends),
                "REJECTED" if same_direction => {
                    // Allow retry after a rejection: reopen the same request.
                    FriendRequestRepo::reset_request_to_pending(pool, row.id.clone()).await?;
                    return Ok(());
                }
                _ => {}
            }
        }
        FriendRequestRepo::insert_friend_request(pool, request).await
    }

    pub async fn reject_friend_request2(pool:&PgPool, frid:String, owner_username:String)->Result<(),Box<dyn Error>>{
        FriendRequestRepo::delete_friend_request(pool, frid, owner_username).await
    }

    pub async fn accept_friend_request(pool:&PgPool, request_id:String, owner_user_name:String)->Result<FriendRequest,Box<dyn Error>> {
        FriendRequestRepo::accept_friend_request(pool, request_id, owner_user_name).await
    }

    pub async fn unfriend(pool:&PgPool, user_a:String, user_b:String)->Result<(),Box<dyn Error>> {
        FriendRequestRepo::unfriend(pool, user_a, user_b).await
    }

    pub async fn are_friends(pool:&PgPool, user_a:String, user_b:String)->Result<bool, ServiceError> {
        FriendRequestRepo::are_friends(pool, user_a, user_b).await
    }

}