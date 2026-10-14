use std::error::Error;

use sqlx::PgPool;

use crate::feed::models::{
    Comment, PollData, Post, PostFeed, PostFeedWithPoll, PostNotificationTarget,
    PostWithComments, PostWithCommentsAndPoll,
};
use crate::feed::repository::PostRepo;

pub struct PostService {}

impl PostService {
    pub async fn create_post(pool: &PgPool, post: Post) -> Result<(), Box<dyn Error>> {
        PostRepo::create_post(pool, post).await
    }

    pub async fn create_post_with_poll(
        pool: &PgPool,
        post: Post,
        options: Option<&[String]>,
    ) -> Result<(), Box<dyn Error>> {
        PostRepo::create_post_with_poll(pool, &post, options).await
    }

    pub async fn create_comment(pool: &PgPool, comment: Comment) -> Result<(), Box<dyn Error>> {
        PostRepo::create_comment(pool, comment).await
    }

    pub async fn toggle_like(
        pool: &PgPool,
        post_id: String,
        user_name: String,
    ) -> Result<bool, Box<dyn Error>> {
        PostRepo::toggle_like(pool, post_id, user_name).await
    }

    pub async fn get_post_owner(
        pool: &PgPool,
        post_id: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        PostRepo::get_post_owner(pool, post_id).await
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

    pub async fn create_poll_options(
        pool: &PgPool,
        post_id: &str,
        options: &[String],
    ) -> Result<(), Box<dyn Error>> {
        PostRepo::create_poll_options(pool, post_id, options).await
    }

    pub async fn get_poll(
        pool: &PgPool,
        post_id: &str,
        viewer: &str,
    ) -> Result<Option<PollData>, Box<dyn Error>> {
        PostRepo::get_poll_for_post(pool, post_id, viewer).await
    }

    pub async fn vote_in_poll(
        pool: &PgPool,
        post_id: String,
        option_id: String,
        user_name: String,
    ) -> Result<PollData, Box<dyn Error>> {
        PostRepo::vote_in_poll(pool, &post_id, &option_id, &user_name).await
    }

    async fn attach_polls(
        pool: &PgPool,
        posts: Vec<PostFeed>,
        viewer: &str,
    ) -> Result<Vec<PostFeedWithPoll>, Box<dyn Error>> {
        let mut out = Vec::with_capacity(posts.len());
        for p in posts {
            let poll = PostRepo::get_poll_for_post(pool, &p.id, viewer)
                .await
                .unwrap_or(None);
            let mut enriched = PostFeedWithPoll::from(p);
            enriched.poll = poll;
            out.push(enriched);
        }
        Ok(out)
    }

    pub async fn get_all_post_with_polls(
        pool: &PgPool,
        viewer: &str,
    ) -> Result<Vec<PostFeedWithPoll>, Box<dyn Error>> {
        let posts = PostRepo::get_all_post(pool).await?;
        Self::attach_polls(pool, posts, viewer).await
    }

    pub async fn get_all_my_posts_with_polls(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<PostFeedWithPoll>, Box<dyn Error>> {
        let posts = PostRepo::get_all_my_posts(pool, user_name.clone()).await?;
        Self::attach_polls(pool, posts, &user_name).await
    }

    pub async fn get_user_posts_with_polls(
        pool: &PgPool,
        user_name: String,
        viewer: String,
    ) -> Result<Vec<PostFeedWithPoll>, Box<dyn Error>> {
        let posts = PostRepo::get_user_posts(pool, user_name).await?;
        Self::attach_polls(pool, posts, &viewer).await
    }

    pub async fn get_single_post_with_polls(
        pool: &PgPool,
        id: String,
        viewer: &str,
    ) -> Result<PostWithCommentsAndPoll, Box<dyn Error>> {
        let single = PostRepo::get_single_post(pool, id.clone()).await?;
        let poll = PostRepo::get_poll_for_post(pool, &id, viewer)
            .await
            .unwrap_or(None);
        let mut enriched = PostFeedWithPoll::from(single.post);
        enriched.poll = poll;
        Ok(PostWithCommentsAndPoll {
            post: enriched,
            comments: single.comments,
        })
    }
}
