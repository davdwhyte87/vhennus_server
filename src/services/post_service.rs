use std::error::Error;
use chrono::NaiveDateTime;
use futures::StreamExt;
// use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;
use serde_derive::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::models::comment::Comment;
use crate::models::post::Post;
use crate::models::likes::Like;
pub struct  PostService{

}

#[derive(Serialize, Debug,  Deserialize, Clone,)]
pub struct PostFeed{
    pub id: String,
    pub image: Option<String>,
    pub text:String,
    pub created_at:NaiveDateTime,
    pub updated_at:NaiveDateTime,
    pub user_name:String,
    pub name:Option<String>,
    pub profile_image:Option<String>,
    pub like_count:Option<i64>,
    pub comment_count:Option<i64>,
}

#[derive(Serialize, Debug,  Deserialize, Clone)]
pub struct FeedComment{
    pub id:String,
    pub text:String,
    pub user_name:String,
    pub created_at:NaiveDateTime,
}

#[derive(Serialize, Debug,  Deserialize, Clone,)]
pub struct PostWithComments{
    pub post:PostFeed,
    pub comments:Vec<Comment>,
}

impl PostService {
    pub async fn create_post(pool:&PgPool, post:Post)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(Post,
            "INSERT INTO posts (id,text,image,created_at,updated_at, user_name) 
             VALUES ($1,$2,$3,$4,$5,$6)",
            post.id,post.text,post.image,post.created_at,post.updated_at,post.user_name
        ).execute(pool).await?;
        Ok(())
    }

    pub async fn create_comment(pool:&PgPool, comment:Comment)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(Comment,
            "INSERT INTO comments (id,text,user_name,created_at,post_id) 
             VALUES ($1,$2,$3,$4,$5)",
            comment.id,comment.text,comment.user_name,comment.created_at,comment.post_id
        ).execute(pool).await?;
        
        Ok(())
    }
    
    pub async fn like_post(pool:&PgPool, post_id:String, user_name:String)->Result<(), Box<dyn Error>>{
        let res = sqlx::query_as!(Like,
            "INSERT INTO likes (user_name, post_id) 
            VALUES ($1,$2)", user_name,post_id)
            .execute(pool).await?;
        Ok(())
    }


    pub async fn get_all_post(pool:&PgPool)->Result<Vec<PostFeed>, Box<dyn Error>>{
        let posts = sqlx::query_as!(PostFeed, 
        r#"
                SELECT
                    posts.id,
                    posts.image,
                    posts.text,
                    posts.created_at,
                    posts.updated_at,
                    posts.user_name,
                    profiles.name,
                    profiles.image AS profile_image,
                    COUNT(DISTINCT likes.user_name) AS like_count,
                    COUNT(DISTINCT comments.id) AS comment_count
                FROM posts
                INNER JOIN profiles ON profiles.user_name = posts.user_name
                LEFT JOIN likes ON likes.post_id = posts.id
                LEFT JOIN comments ON comments.post_id = posts.id  -- Ensure this join is present
                GROUP BY
                    posts.id,
                    posts.image,
                    posts.text,
                    posts.created_at,
                    posts.updated_at,
                    posts.user_name,
                    profiles.name,
                    profiles.image
                    "#).fetch_all(pool).await?;


        Ok(posts)
    }


    pub async fn get_all_my_posts(pool:&PgPool, user_name:String)->Result<Vec<PostFeed>, Box<dyn Error>>{
        let posts = sqlx::query_as!(PostFeed, 
        r#"
                SELECT
                    posts.id,
                    posts.image,
                    posts.text,
                    posts.created_at,
                    posts.updated_at,
                    posts.user_name,
                    profiles.name,
                    profiles.image AS profile_image,
                    COUNT(DISTINCT likes.user_name) AS like_count,
                    COUNT(DISTINCT comments.id) AS comment_count
                FROM posts
             
                INNER JOIN profiles ON profiles.user_name = posts.user_name
                LEFT JOIN likes ON likes.post_id = posts.id
                LEFT JOIN comments ON comments.post_id = posts.id  
                WHERE posts.user_name = $1
                GROUP BY
                    posts.id,
                    posts.image,
                    posts.text,
                    posts.created_at,
                    posts.updated_at,
                    posts.user_name,
                    profiles.name,
                    profiles.image
                    "#, user_name).fetch_all(pool).await?;


        Ok(posts)
    }

    pub async fn get_single_post(pool:&PgPool, id:String)->Result<PostWithComments, Box<dyn Error>>{
        let posts = sqlx::query_as!(PostFeed, 
        r#"
                SELECT
                    posts.id,
                    posts.image,
                    posts.text,
                    posts.created_at,
                    posts.updated_at,
                    posts.user_name,
                    profiles.name,
                    profiles.image AS profile_image,
                    COUNT(DISTINCT likes.user_name) AS like_count,
                    COUNT(DISTINCT comments.id) AS comment_count
                FROM posts
             
                INNER JOIN profiles ON profiles.user_name = posts.user_name
                LEFT JOIN likes ON likes.post_id = posts.id
                LEFT JOIN comments ON comments.post_id = posts.id  
                WHERE posts.id = $1
                GROUP BY
                    posts.id,
                    posts.image,
                    posts.text,
                    posts.created_at,
                    posts.updated_at,
                    posts.user_name,
                    profiles.name,
                    profiles.image
                    "#, id.clone()).fetch_one(pool).await?;


        let comments_query = r#"
                SELECT id, text, user_name, created_at
                FROM comments
                WHERE post_id = $1
                ORDER BY created_at ASC
            "#;
        
        let comments = sqlx::query_as!(Comment, "
            SELECT * FROM comments WHERE post_id = $1
            ORDER BY created_at ASC
            ", id).fetch_all(pool).await?;

            let post_with_comments = PostWithComments{
                post: posts,
                comments: comments,
            };

            Ok(post_with_comments)
    }

    // pub async fn update_post(db:&Database, post:Post)->Result<(), Box<dyn Error>>{
    //     let collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
    //     let update_doc = doc! {
    //         "$set":
    //            doc!{
    //               "likes": post.likes.to_owned(),
    //            }
    //     };
    //
    //     let filter = doc! {"id":post.id};
    //
    //    let mut results = collection.update_one(filter,update_doc).await;
    //    match results {
    //        Ok(_)=>{
    //
    //        },
    //        Err(err)=>{
    //         log::error!(" error updating post {}", err.to_string());
    //         return Err(err.into());
    //        }
    //    }
    //
    //
    //    return Ok(());
    // }



}