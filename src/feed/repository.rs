use std::error::Error;

use sqlx::PgPool;

use crate::feed::models::{
    Comment, FeedComment, Like, Post, PostFeed, PostNotificationTarget, PostWithComments,
};

pub struct PostRepo {}

impl PostRepo {
    pub async fn create_post(pool: &PgPool, post: Post) -> Result<(), Box<dyn Error>> {
        sqlx::query_as!(
            Post,
            "INSERT INTO posts (id,text,image,created_at,updated_at, user_name)
             VALUES ($1,$2,$3,$4,$5,$6)",
            post.id,
            post.text,
            post.image,
            post.created_at,
            post.updated_at,
            post.user_name
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn create_comment(pool: &PgPool, comment: Comment) -> Result<(), Box<dyn Error>> {
        sqlx::query_as!(
            Comment,
            "INSERT INTO comments (id,text,user_name,created_at,post_id)
             VALUES ($1,$2,$3,$4,$5)",
            comment.id,
            comment.text,
            comment.user_name,
            comment.created_at,
            comment.post_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn like_post(
        pool: &PgPool,
        post_id: String,
        user_name: String,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query_as!(
            Like,
            "INSERT INTO likes (user_name, post_id)
            VALUES ($1,$2)",
            user_name,
            post_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn toggle_like(
        pool: &PgPool,
        post_id: String,
        user_name: String,
    ) -> Result<(), Box<dyn Error>> {
        let existing_like = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM likes WHERE user_name = $1 AND post_id = $2",
            user_name,
            post_id
        )
        .fetch_one(pool)
        .await?;

        if existing_like.unwrap_or(0) > 0 {
            sqlx::query!(
                "DELETE FROM likes WHERE user_name = $1 AND post_id = $2",
                user_name,
                post_id
            )
            .execute(pool)
            .await?;
        } else {
            sqlx::query!(
                "INSERT INTO likes (user_name, post_id) VALUES ($1, $2)",
                user_name,
                post_id
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_all_post(pool: &PgPool) -> Result<Vec<PostFeed>, Box<dyn Error>> {
        let posts = sqlx::query_as!(
            PostFeed,
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
                ORDER BY posts.created_at DESC
                    "#
        )
        .fetch_all(pool)
        .await?;

        Ok(posts)
    }

    pub async fn get_all_my_posts(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<PostFeed>, Box<dyn Error>> {
        let posts = sqlx::query_as!(
            PostFeed,
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
                    "#,
            user_name
        )
        .fetch_all(pool)
        .await?;

        Ok(posts)
    }

    pub async fn get_user_posts(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<PostFeed>, Box<dyn Error>> {
        let posts = sqlx::query_as!(
            PostFeed,
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
                    "#,
            user_name
        )
        .fetch_all(pool)
        .await?;
        Ok(posts)
    }

    pub async fn get_single_post(
        pool: &PgPool,
        id: String,
    ) -> Result<PostWithComments, Box<dyn Error>> {
        let posts = sqlx::query_as!(
            PostFeed,
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
                    "#,
            id.clone()
        )
        .fetch_one(pool)
        .await?;

        let comments = sqlx::query_as!(
            Comment,
            "
            SELECT * FROM comments WHERE post_id = $1
            ORDER BY created_at ASC
            ",
            id
        )
        .fetch_all(pool)
        .await?;

        let post_with_comments = PostWithComments {
            post: posts,
            comments: comments,
        };

        Ok(post_with_comments)
    }

    pub async fn get_last_1hr_comments(
        pool: &PgPool,
    ) -> Result<Vec<PostNotificationTarget>, Box<dyn Error>> {
        let data = sqlx::query_as!(
            PostNotificationTarget,
            r#"SELECT
                 posts.id,
                 posts.user_name,
                 profiles.app_f_token AS token
             FROM comments
             JOIN posts ON  comments.post_id = posts.id
             JOIN profiles ON posts.user_name = profiles.user_name
             WHERE comments.created_at >= NOW() - INTERVAL '1 hour'
       "#
        )
        .fetch_all(pool)
        .await?;

        return Ok(data);
    }
}
