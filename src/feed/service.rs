use std::error::Error;

use sqlx::PgPool;

use crate::feed::models::{
    Comment, Post, PostFeed, PostNotificationTarget, PostWithComments,
};
use crate::feed::repository::PostRepo;

pub struct PostService {}

impl PostService {
    pub async fn create_post(pool: &PgPool, post: Post) -> Result<(), Box<dyn Error>> {
        PostRepo::create_post(pool, post).await
    }

    pub async fn create_comment(pool: &PgPool, comment: Comment) -> Result<(), Box<dyn Error>> {
        PostRepo::create_comment(pool, comment).await
    }

    pub async fn toggle_like(
        pool: &PgPool,
        post_id: String,
        user_name: String,
    ) -> Result<(), Box<dyn Error>> {
        PostRepo::toggle_like(pool, post_id, user_name).await
    }

    pub async fn get_all_post(pool: &PgPool) -> Result<Vec<PostFeed>, Box<dyn Error>> {
        PostRepo::get_all_post(pool).await
    }

    pub async fn get_all_my_posts(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<PostFeed>, Box<dyn Error>> {
        PostRepo::get_all_my_posts(pool, user_name).await
    }

    pub async fn get_user_posts(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<PostFeed>, Box<dyn Error>> {
        PostRepo::get_user_posts(pool, user_name).await
    }

    pub async fn get_single_post(
        pool: &PgPool,
        id: String,
    ) -> Result<PostWithComments, Box<dyn Error>> {
        PostRepo::get_single_post(pool, id).await
    }

    pub async fn get_last_1hr_comments(
        pool: &PgPool,
    ) -> Result<Vec<PostNotificationTarget>, Box<dyn Error>> {
        PostRepo::get_last_1hr_comments(pool).await
    }
}
