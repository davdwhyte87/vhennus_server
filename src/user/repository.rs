use std::error::Error;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::shared::error::ServiceError;
use crate::user::models::{Friend, FriendRequest, FriendRequestWithProfile, User};

pub struct UserRepo;

impl UserRepo {
    pub async fn insert_user_with_profile(pool:&PgPool, user:User, code:i32)->Result<(), Box<dyn Error>>{
        let mut tx: Transaction<'_, Postgres> = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err("Failed to start transaction".into()),
        };
        // check if the user exists
        if Self::user_email_exists(pool,user.email.clone().unwrap().as_str()).await.unwrap(){
            return Err(Box::from("UserEmail already exists"));
        }

        //create user
        let user_insert = sqlx::query_as!(User,
            "INSERT INTO users (id, user_name, email, password_hash, code) 
             VALUES ($1, $2, $3, $4, $5)",
            user.id.clone(),
            user.user_name.clone(),
            user.email.clone(),
            user.password_hash.clone(),
            code.clone()
        )
            .execute(&mut *tx)
            .await;
        
        if user_insert.is_err(){
            let _ = tx.rollback().await;
            return Err(user_insert.err().unwrap().description().into());
        }
        let profile_insert = sqlx::query!(
            "INSERT INTO profiles (id,user_name) 
             VALUES ($1, $2)",
            Uuid::new_v4().to_string(),
            user.user_name.clone(),
            
        )
            .execute(&mut *tx)
            .await;

        if profile_insert.is_err() {
            let _ = tx.rollback().await;
            return Err("Failed to create profile".into());
        }
        
        if let Err(err) = tx.commit().await {
            log::error!("Failed to commit transaction: {}", err);
            return Err("Transaction commit failed".into());
        }
        return Ok(());
    }

    pub async fn get_by_username(pool:&PgPool, username:String)->Result<Option<User>, Box<dyn Error>>{
        let user =match  sqlx::query_as!(User, 
        "SELECT * FROM users WHERE user_name = $1 ", username.clone() )
            .fetch_optional(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(user);
    }

    pub async fn get_all(pool:&PgPool)->Result<Vec<User>, Box<dyn Error>>{
        let user =match  sqlx::query_as!(User,
        "SELECT * FROM users")
            .fetch_all(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(user);
    }

    pub async fn get_by_email(pool:&PgPool, email:String)->Result<Option<User>, Box<dyn Error>>{
        let user =match  sqlx::query_as!(User, 
        "SELECT * FROM users WHERE email = $1 ", email.clone() )
            .fetch_optional(pool)
            .await{
            Ok(opt) => opt,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        return Ok(user);
    }

    pub async fn confirm_user_email(pool:&PgPool, email:String, code:String)->Result<(), Box<dyn Error>>{
        
        // get user 
        let user =match Self::get_by_email(pool,email.clone()).await{
            Ok(user) => { 
                match user {
                    Some(user) => {user},
                    None=>{
                        return Err(Box::from("User not found"));
                    }
                }
            },
            Err(err)=>{
                log::error!("Error getting user");
                return Err(err);
            }
        };
        if code != user.code.unwrap().to_string(){
            return Err(Box::from("Wrong code"));
        }
        let res = sqlx::query_as!(User,
        "UPDATE users SET email_confirmed=$1 WHERE email = $2 ",true, email.clone() )
            .execute(pool).await?;

        Ok(())
    }
    
    pub async fn update_user_code(pool:&PgPool, email:String, code:i32)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(User,
        "UPDATE users SET code=$1 WHERE email = $2 ",code.clone(), email.clone() )
            .execute(pool).await?;
        Ok(())
    }

    pub async fn update(pool:&PgPool, user:User)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(User,
        "UPDATE users SET 
                 code=COALESCE($2,code),
                 password_hash = COALESCE($3, password_hash)
        WHERE user_name = $1
        ",user.user_name.clone(), user.code.clone(), user.password_hash.clone() )
            
            .execute(pool).await?;
        Ok(())
    }
    
    
    pub async fn delete_user(pool:&PgPool, username:String)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(User,
        "UPDATE users SET is_deleted=$1 WHERE user_name = $2 ",true, username.clone() )
            .execute(pool).await?;
        
        Ok(())
    }

    pub async fn user_email_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)",
        email
        )
            .fetch_one(pool)
            .await?;

        Ok(exists.unwrap_or(false))
    }
}

pub struct FriendRequestRepo;

impl FriendRequestRepo {
    pub async fn get_user_friend_request(pool:&PgPool, other_user_name:String)->Result<Vec<FriendRequestWithProfile>, Box<dyn Error>>{
        let chat_pair = sqlx::query_as!(FriendRequestWithProfile, "
            SELECT fr.id,fr.user_name,fr.requester, fr.status,fr.created_at,
            p.bio,p.name,p.image    
            FROM friend_requests fr
            INNER JOIN profiles p ON p.user_name = fr.requester
            WHERE fr.user_name = $1 AND fr.status = $2
            " , other_user_name, "PENDING")
            .fetch_all(pool).await?;
        Ok(chat_pair)
    }

    pub async fn get_single_friend_request(pool:&PgPool, id:String)->Result<FriendRequest, Box<dyn Error>>{
        let fr = sqlx::query_as!(FriendRequest, "
            SELECT *    
            FROM friend_requests 
            WHERE id = $1
            " , id)
            .fetch_one(pool).await?;
        Ok(fr)
    }

    pub async fn get_single_friend_request_by_users(pool:&PgPool, requester:String, user:String)->Result<Option<FriendRequest>, ServiceError>{
        let fr = sqlx::query_as!(FriendRequest, "
            SELECT *    
            FROM friend_requests 
            WHERE requester = $1 AND user_name=$2
            " , requester, user)
            .fetch_optional(pool).await?;
        Ok(fr)
    }

    pub async fn insert_friend_request(pool:&PgPool, request:FriendRequest)->Result<(), ServiceError>{
        let res = sqlx::query_as!(FriendRequest, "
          INSERT INTO friend_requests (id,user_name,requester, status, created_at, updated_at)
          VALUES ($1,$2,$3,$4,$5,$6)",
            request.id,request.user_name,request.requester,request.status,request.created_at,request.updated_at
        ).execute(pool).await?;
        Ok(())
    }

    pub async fn delete_friend_request(pool:&PgPool, frid:String, owner_username:String)->Result<(),Box<dyn Error>>{
        let res = sqlx::query_as!(FriendRequest, 
            "DELETE FROM friend_requests  WHERE id = $1 AND user_name = $2",
            frid, owner_username
        ).execute(pool).await?;
        return Ok(())
    }

    pub async fn accept_friend_request(pool:&PgPool, request_id:String, owner_user_name:String)->Result<FriendRequest,Box<dyn Error>> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
        // get friend request 
        let fr = Self::get_single_friend_request(pool, request_id.clone()).await?;
        
        // update friend request 
        let res = sqlx::query_as!(FriendRequest, 
            "UPDATE friend_requests SET status =COALESCE($3,status) WHERE id = $1 AND user_name = $2",
            request_id, owner_user_name,"ACCEPTED"
        ).execute(&mut *tx).await?;
     
        if res.rows_affected() == 0 {
            tx.rollback().await?;
            return  return Err("Failed to update friend request".into());
        }
        
        let friend = Friend{
            id: 0,
            user_username: "".to_string(),
            friend_username: "".to_string(),
        };
        let result = sqlx::query_as!(Friend, "
            INSERT INTO friends (user_username,friend_username)
            VALUES ($1,$2)
            ",owner_user_name, fr.requester )
            .execute(&mut *tx).await?;
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err("Failed to create friend".into());
        }
        tx.commit().await?;
        return Ok(fr)
    }
}