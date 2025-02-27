use diesel::pg::Pg;
use diesel::sql_types::Text;
use std::error::Error;
use chrono::NaiveDateTime;
use diesel::{sql_query, ExpressionMethods, Insertable, JoinOnDsl, QueryDsl, Queryable, QueryableByName, RunQueryDsl, Selectable};
use diesel::dsl::{count, count_distinct, sql};
use futures::StreamExt;
// use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;
use serde_derive::{Deserialize, Serialize};
use crate::DbPool;
use crate::models::{buy_order::BuyOrder, comment::Comment, payment_method::PaymentMethodData, post::Post, sell_order::SellOrder};
use crate::schema::comments::dsl::comments;
use crate::schema::posts::dsl::posts;
use diesel::sql_types::{Integer, Nullable, Timestamp, BigInt};
pub const POST_SERVICE_COLLECTION:&str = "Post";
pub const COMMENT_COLLECTION:&str = "Comment";
use diesel::prelude::*;
pub struct  PostService{

}

#[derive(Serialize, Debug,  Deserialize, Clone, QueryableByName, Queryable)]
#[diesel(check_for_backend(Pg))]
pub struct PostFeed{
    #[sql_type = "Text"]
    pub id: String,
    #[sql_type = "Nullable<Text>"]
    pub image: Option<String>,
    #[sql_type = "Text"]
    pub text:String,
    #[sql_type = "Timestamp"]
    pub created_at:NaiveDateTime,
    #[sql_type = "Timestamp"]
    pub updated_at:NaiveDateTime,
    #[sql_type = "Text"]
    pub user_name:String,
    #[sql_type = "Nullable<Text>"]
    pub name:Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub profile_image:Option<String>,
    #[sql_type = "BigInt"]
    pub like_count:i64,
    #[sql_type = "BigInt"]
    pub comment_count:i64,
}

#[derive(Serialize, Debug,  Deserialize, Clone, QueryableByName, Queryable)]
#[diesel(check_for_backend(Pg))]
pub struct FeedComment{
    #[sql_type = "Text"]
    pub id:String,
    #[sql_type = "Text"]
    pub text:String,
    #[sql_type = "Text"]
    pub user_name:String,
    #[sql_type = "Timestamp"]
    pub created_at:NaiveDateTime,
}

#[derive(Serialize, Debug,  Deserialize, Clone,)]
pub struct PostWithComments{
    pub post:PostFeed,
    pub comments:Vec<FeedComment>,
}

impl PostService {
    pub async fn create_post(pool:&DbPool, post:Post)->Result<(), Box<dyn Error>>{
        // create post
        let xpool = pool.clone();
        let res = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("couldn't get db connection from pool");
            diesel::insert_into(posts).values(&post).execute(&mut conn)
        }).await??;
        if res == 0{
            return Err(Box::from("Error creating post"));
        }
        Ok(())
    }

    pub async fn create_comment(pool:&DbPool, comment:Comment)->Result<(), Box<dyn Error>>{
        let xpool = pool.clone();
        let res = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("couldn't get db connection from pool");
            diesel::insert_into(comments).values(&comment).execute(&mut conn)
        }).await??;
        if res == 0{
            return Err(Box::from("Error creating comment"));
        }
        Ok(())
    }


    pub async fn get_all_post(pool:&DbPool)->Result<Vec<PostFeed>, Box<dyn Error>>{

        let xpool = pool.clone();
        use crate::schema::profiles as sprofiles;
        use crate::schema::posts as sposts;
        use crate::schema::likes as slikes;
        use crate::schema::comments as scomments;

        let query = r#"
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
                    "#;
        let res = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("couldn't get db connection from pool");
            // posts.inner_join(sprofiles::table.on(sprofiles::user_name.eq(sposts::user_name)))
            //     .left_join(slikes::table.on(slikes::user_name.eq(sposts::user_name)))
            //     .left_join(scomments::table.on(scomments::post_id.eq(sposts::id)))
            //     .group_by((sposts::id,
            //                sposts::image,
            //                sposts::text,
            //     ))
            //     .select((
            //         sposts::id,
            //         sposts::image,
            //         sposts::text,
            //         sposts::created_at,
            //         sposts::updated_at,
            //         sposts::user_name,
            //         sprofiles::name,
            //         sprofiles::image,
            //         sql::<diesel::sql_types::BigInt>("COUNT(DISTINCT likes.user_name)"),  // ✅ Fixing count distinct
            //         sql::<diesel::sql_types::BigInt>("COUNT(DISTINCT comments.id)")
            //     ))
            //     .load::<PostFeed>(&mut conn);

            let results =
                match sql_query(query).load::<PostFeed>(&mut conn){
                    Ok(v) => v,
                    Err(err)=>{
                        return Err(Box::new(err))
                    }
                };

            Ok(results)
            
        }).await??;

        Ok(res)
    }


    pub async fn get_all_my_posts(pool:&DbPool, user_name:String)->Result<Vec<PostFeed>, Box<dyn Error>>{

        let xpool = pool.clone();


        let query = r#"
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
                    "#;
        let res = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("couldn't get db connection from pool");
            let results =
                match sql_query(query)
                    .bind::<Text, _>(user_name.clone())
                    .load::<PostFeed>(&mut conn){
                    Ok(v) => v,
                    Err(err)=>{
                        return Err(Box::new(err))
                    }
                };

            Ok(results)

        }).await??;

        Ok(res)
    }

    pub async fn get_single_post(pool:&DbPool, id:String)->Result<PostWithComments, Box<dyn Error>>{
        let xpool = pool.clone();
        let query = r#"
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
                    "#;
        let res = actix_web::web::block(move || {
            let mut conn = xpool.get().expect("couldn't get db connection from pool");
            let mut conn2 = xpool.get().expect("couldn't get db connection from pool");
            let results =
                match sql_query(query)
                    .bind::<Text, _>(id.clone())
                    .get_result::<PostFeed>(&mut conn){
                    Ok(v) => v,
                    Err(err)=>{
                        return Err(Box::new(err))
                    }
                };
            let comments_query = r#"
                SELECT id, text, user_name, created_at
                FROM comments
                WHERE post_id = $1
                ORDER BY created_at ASC
            "#;

            let other_comments = match  sql_query(comments_query)
                .bind::<Text, _>(id.clone())
                .load::<FeedComment>(&mut conn2){
                Ok(v) => v,
                Err(err)=>{
                    return Err(Box::new(err))
                }
            };
            
            let post_with_comments = PostWithComments{
                post: results,
                comments: other_comments,
            };

            Ok(post_with_comments)

        }).await??;

        Ok(res)
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