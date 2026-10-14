use std::error::Error;

use sqlx::PgPool;

use crate::feed::models::{
    Comment, FeedComment, Like, PollData, PollOption, PollOptionResult, Post, PostFeed,
    PostNotificationTarget, PostWithComments,
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
    ) -> Result<bool, Box<dyn Error>> {
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
            Ok(false)
        } else {
            sqlx::query!(
                "INSERT INTO likes (user_name, post_id) VALUES ($1, $2)",
                user_name,
                post_id
            )
            .execute(pool)
            .await?;
            Ok(true)
        }
    }

    /// Lightweight owner lookup for notification hooks.
    pub async fn get_post_owner(
        pool: &PgPool,
        post_id: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        #[derive(sqlx::FromRow)]
        struct Owner {
            user_name: String,
        }
        let row: Option<Owner> = sqlx::query_as::<_, Owner>(
            "SELECT user_name FROM posts WHERE id = $1",
        )
        .bind(post_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|r| r.user_name))
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

    // ---------------- Polls (runtime queries, no compile-time DB check) ----------------

    /// Insert a post and its poll options atomically. If the poll insert
    /// fails the post is rolled back too (no orphan posts).
    pub async fn create_post_with_poll(
        pool: &PgPool,
        post: &Post,
        options: Option<&[String]>,
    ) -> Result<(), Box<dyn Error>> {
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO posts (id,text,image,created_at,updated_at,user_name)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(&post.id)
        .bind(&post.text)
        .bind(&post.image)
        .bind(post.created_at)
        .bind(post.updated_at)
        .bind(&post.user_name)
        .execute(&mut *tx)
        .await?;

        if let Some(opts) = options {
            for (idx, text) in opts.iter().enumerate() {
                let id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO poll_options (id, post_id, text, position) VALUES ($1,$2,$3,$4)",
                )
                .bind(&id)
                .bind(&post.id)
                .bind(text)
                .bind(idx as i32)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn create_poll_options(
        pool: &PgPool,
        post_id: &str,
        options: &[String],
    ) -> Result<(), Box<dyn Error>> {
        for (idx, text) in options.iter().enumerate() {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO poll_options (id, post_id, text, position) VALUES ($1,$2,$3,$4)",
            )
            .bind(&id)
            .bind(post_id)
            .bind(text)
            .bind(idx as i32)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    pub async fn get_poll_for_post(
        pool: &PgPool,
        post_id: &str,
        viewer_user_name: &str,
    ) -> Result<Option<PollData>, Box<dyn Error>> {
        let options: Vec<PollOption> =
            sqlx::query_as::<_, PollOption>(
                "SELECT id, post_id, text, position, created_at FROM poll_options WHERE post_id = $1 ORDER BY position ASC",
            )
            .bind(post_id)
            .fetch_all(pool)
            .await?;

        if options.is_empty() {
            return Ok(None);
        }

        #[derive(sqlx::FromRow)]
        struct VoteCount {
            option_id: String,
            count: Option<i64>,
        }
        let counts: Vec<VoteCount> = sqlx::query_as::<_, VoteCount>(
            "SELECT option_id, COUNT(*) AS count FROM poll_votes WHERE post_id = $1 GROUP BY option_id",
        )
        .bind(post_id)
        .fetch_all(pool)
        .await?;

        #[derive(sqlx::FromRow)]
        struct ViewerVote {
            option_id: String,
        }
        let viewer_vote: Option<ViewerVote> = sqlx::query_as::<_, ViewerVote>(
            "SELECT option_id FROM poll_votes WHERE post_id = $1 AND user_name = $2",
        )
        .bind(post_id)
        .bind(viewer_user_name)
        .fetch_optional(pool)
        .await?;

        let mut results: Vec<PollOptionResult> = options
            .into_iter()
            .map(|o| {
                let c = counts
                    .iter()
                    .find(|v| v.option_id == o.id)
                    .and_then(|v| v.count)
                    .unwrap_or(0);
                PollOptionResult {
                    id: o.id,
                    text: o.text,
                    position: o.position,
                    vote_count: c,
                }
            })
            .collect();
        results.sort_by_key(|r| r.position);
        let total: i64 = results.iter().map(|r| r.vote_count).sum();

        Ok(Some(PollData {
            options: results,
            total_votes: total,
            user_voted_option_id: viewer_vote.map(|v| v.option_id),
            has_poll: true,
        }))
    }

    pub async fn vote_in_poll(
        pool: &PgPool,
        post_id: &str,
        option_id: &str,
        user_name: &str,
    ) -> Result<PollData, Box<dyn Error>> {
        // Ensure the option belongs to the post.
        let owner: Option<(String,)> = sqlx::query_as(
            "SELECT post_id FROM poll_options WHERE id = $1",
        )
        .bind(option_id)
        .fetch_optional(pool)
        .await?;
        match owner {
            Some((owner_post_id,)) if owner_post_id == post_id => {}
            _ => return Err("Poll option does not belong to this post".into()),
        }

        let vote_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO poll_votes (id, post_id, option_id, user_name)
               VALUES ($1,$2,$3,$4)
               ON CONFLICT (post_id, user_name)
               DO UPDATE SET option_id = EXCLUDED.option_id, created_at = NOW()"#,
        )
        .bind(&vote_id)
        .bind(post_id)
        .bind(option_id)
        .bind(user_name)
        .execute(pool)
        .await?;

        let poll = Self::get_poll_for_post(pool, post_id, user_name)
            .await?
            .unwrap_or(PollData {
                options: vec![],
                total_votes: 0,
                user_voted_option_id: None,
                has_poll: true,
            });
        Ok(poll)
    }
}
