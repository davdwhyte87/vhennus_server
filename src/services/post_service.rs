use std::error::Error;

use futures::StreamExt;
use lettre::transport::smtp::commands::Data;
use mongodb::{bson::{doc, from_document}, results::InsertOneResult, Database};
use r2d2_mongodb::mongodb::coll;

use crate::models::{buy_order::BuyOrder, comment::Comment, payment_method::PaymentMethodData, post::Post, sell_order::SellOrder};

pub const POST_SERVICE_COLLECTION:&str = "Post";
pub const COMMENT_COLLECTION:&str = "Comment";

pub struct  PostService{

}

impl PostService {
    pub async fn create_post(db:&Database, post:&Post)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
  
        let res_sell_order =collection.insert_one(post).await;

        let res = match res_sell_order {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting into db  {}", err.to_string());
                return Err(err.into())
            }
        };
        Ok(res)
    }

    pub async fn create_comment(db:&Database, comment:&Comment)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<Comment>(COMMENT_COLLECTION);
  
   

        // update post 
        let post_collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
        // get the post 
        let mut post = match post_collection.find_one(doc! {"id":comment.post_id.to_owned()}).await{
            Ok(data)=>{
                match data{
                    Some(data)=>{data},
                    None=>{
                        log::info!(" post not found");
                        return Err(Box::from("post not found"))
                    }
                }
            },
            Err(err)=>{
                log::error!(" error fetching post {}", err.to_string());
                return Err(err.into())
            }
        };

        let res_sell_order =collection.insert_one(comment).await;

        let res = match res_sell_order {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting comment data into db {}", err.to_string());
                return Err(err.into())
            }
        };
        // update post
        post.comments_ids.push(comment.id.to_owned());
        match post_collection.update_one(doc! {"id": comment.post_id.to_owned()},
        doc! {"$set":doc! {
            "comments_ids":post.comments_ids.to_owned()
        }}).await{
            Ok(data)=>{},
            Err(err)=>{
                log::error!(" error updating post {}", err.to_string());
                return Err(err.into())
            }
        }

        Ok(res)
    }

    pub async fn get_all_post(db:&Database)->Result<Vec<Post>, Box<dyn Error>>{
        let collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "Comment",
                  "localField": "comments_ids",
                  "foreignField": "id",
                  "as": "comments"
               }
        };

       let mut results = collection.aggregate(vec![lookup_2]).await?;
       let mut posts:Vec<Post> = Vec::new();
       while let Some(result) = results.next().await{
           let data: Post= from_document(result?)?;
           posts.push(data);
       }
       return Ok(posts);
    }


    pub async fn get_all_my_posts(db:&Database, user_name:String)->Result<Vec<Post>, Box<dyn Error>>{
        let collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "Comment",
                  "localField": "comments_ids",
                  "foreignField": "id",
                  "as": "comments"
               }
        };

        let filter = doc! {
            "$match":{"user_name":user_name}
        };

       let mut results = collection.aggregate(vec![filter,lookup_2]).await?;
       let mut posts:Vec<Post> = Vec::new();
       while let Some(result) = results.next().await{
           let data: Post= from_document(result?)?;
           posts.push(data);
       }
       return Ok(posts);
    }

    pub async fn get_single_post(db:&Database, id:String)->Result<Post, Box<dyn Error>>{
        let collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "Comment",
                  "localField": "comments_ids",
                  "foreignField": "id",
                  "as": "comments"
               }
        };

        let filter = doc! {"$match":doc! {"id":id}};

       let mut results = collection.aggregate(vec![filter,lookup_2]).await?;
       let mut post:Post = Post::default();
       while let Some(result) = results.next().await{
           let data: Post= from_document(result?)?;
           post = data
       }
       return Ok(post);
    }

    pub async fn update_post(db:&Database, post:Post)->Result<(), Box<dyn Error>>{
        let collection = db.collection::<Post>(POST_SERVICE_COLLECTION);
        let update_doc = doc! {
            "$set":
               doc!{
                  "likes": post.likes.to_owned(),
               }
        };

        let filter = doc! {"id":post.id};

       let mut results = collection.update_one(filter,update_doc).await;
       match results {
           Ok(_)=>{

           },
           Err(err)=>{
            log::error!(" error updating post {}", err.to_string());
            return Err(err.into());
           }
       }
       
      
       return Ok(());
    }



}