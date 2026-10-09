use std::error::Error;

use sqlx::PgPool;

use crate::chat::models::{Chat, ChatPair, ChatPairView, Circle, UnreadRow};
use crate::shared::error::ServiceError;
use crate::shared::general::get_time_naive;

pub struct ChatRepo {}

impl ChatRepo {
    pub async fn create_chat(pool: &PgPool, chat: &Chat) -> Result<(), Box<dyn Error>> {
        sqlx::query_as!(
            Chat,
            "INSERT INTO chats (id,sender,receiver,message, image,created_at,updated_at, pair_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            chat.id,
            chat.sender,
            chat.receiver,
            chat.message,
            chat.image,
            chat.created_at,
            chat.updated_at,
            chat.pair_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_chats_by_pair_id(
        pool: &PgPool,
        id: String,
    ) -> Result<Vec<Chat>, Box<dyn Error>> {
        let chats = sqlx::query_as!(
            Chat,
            "SELECT * FROM chats WHERE pair_id = $1 ORDER BY created_at ASC, id ASC",
            id
        )
        .fetch_all(pool)
        .await?;
        Ok(chats)
    }
}

pub struct ChatPairRepo {}

impl ChatPairRepo {
    pub async fn create_chat_pair(
        pool: &PgPool,
        chat: &ChatPair,
    ) -> Result<ChatPair, Box<dyn Error>> {
        let res_chat = chat.clone();
        let res = sqlx::query_as!(
            ChatPair,
            "INSERT INTO chat_pairs (id, user1,user2,created_at,updated_at,last_message)
             VALUES ($1, $2, $3, $4, $5, $6)
             ",
            chat.id,
            chat.user1,
            chat.user2,
            chat.created_at,
            chat.updated_at,
            chat.last_message
        )
        .execute(pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(Box::from("Could not create pair"));
        }
        Ok(res_chat)
    }

    pub async fn update_chat_pair(
        pool: &PgPool,
        id: String,
        last_message: String,
    ) -> Result<(), ServiceError> {
        let date = get_time_naive();
        let res = sqlx::query_as!(
            ChatPair,
            "UPDATE chat_pairs
            SET
                last_message =COALESCE($1, last_message),
                updated_at = COALESCE($2, updated_at)
            WHERE id = $3
            ",
            last_message,
            date,
            id
        )
        .execute(pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(ServiceError::NoUpdatedRow);
        }
        Ok(())
    }

    pub async fn find_chat_pair(
        pool: &PgPool,
        xuser1: String,
        xuser2: String,
    ) -> Result<ChatPairView, Box<dyn Error>> {
        let chat_pair = sqlx::query_as!(
            ChatPairView,
            "
             SELECT cp.id, cp.user1, cp.user2, cp.last_message, cp.all_read, cp.created_at,
               cp.updated_at, p1.image AS user1_image, p2.image AS user2_image
             FROM chat_pairs cp
             JOIN profiles p1 ON p1.user_name = cp.user1
             JOIN profiles p2 ON p2.user_name = cp.user2
             WHERE (user1 = $1 AND user2 = $2) OR (user1 = $2 AND user2 = $1)
             ",
            xuser1,
            xuser2
        )
        .fetch_one(pool)
        .await?;
        Ok(chat_pair)
    }

    pub async fn find_chat_pair_by_id(
        pool: &PgPool,
        id: String,
    ) -> Result<ChatPair, Box<dyn Error>> {
        let chat_pair = sqlx::query_as!(ChatPair, "SELECT * FROM chat_pairs WHERE id = $1", id)
            .fetch_one(pool)
            .await?;
        Ok(chat_pair)
    }

    pub async fn get_all_my_chat_pairs(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<ChatPairView>, Box<dyn Error>> {
        let chat_pairs = sqlx::query_as!(
            ChatPairView,
            "
            SELECT
               cp.id, cp.user1, cp.user2, cp.last_message, cp.all_read, cp.created_at,
               cp.updated_at, p1.image AS user1_image, p2.image AS user2_image
            FROM chat_pairs cp
            JOIN profiles p1 ON p1.user_name = cp.user1
            JOIN profiles p2 ON p2.user_name = cp.user2
            WHERE user1 = $1 OR user2 =$1
            ",
            user_name
        )
        .fetch_all(pool)
        .await?;
        Ok(chat_pairs)
    }

    pub async fn mark_pair_read(
        pool: &PgPool,
        pair_id: String,
        user_name: String,
        now: chrono::NaiveDateTime,
    ) -> Result<(), Box<dyn Error>> {
        // NOTE: `now` must come from the app clock (get_time_naive), never
        // SQL NOW(): message timestamps are app-clock naive times, while
        // NOW() is cast through the DB session timezone and can lag/lead them.
        sqlx::query!(
            "INSERT INTO chat_reads (pair_id, user_name, last_read_at)
             VALUES ($1, $2, $3)
             ON CONFLICT (pair_id, user_name)
             DO UPDATE SET last_read_at = $3",
            pair_id, user_name, now
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_unread(
        pool: &PgPool,
        user_name: String,
    ) -> Result<Vec<UnreadRow>, Box<dyn Error>> {
        let rows = sqlx::query_as!(
            UnreadRow,
            "SELECT cp.id AS pair_id,
                    COUNT(c.id) FILTER (
                        WHERE c.sender <> $1
                          AND c.created_at > COALESCE(r.last_read_at, '1970-01-01'::timestamp)
                    ) AS unread
             FROM chat_pairs cp
             LEFT JOIN chats c ON c.pair_id = cp.id
             LEFT JOIN chat_reads r ON r.pair_id = cp.id AND r.user_name = $1
             WHERE cp.user1 = $1 OR cp.user2 = $1
             GROUP BY cp.id",
            user_name
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

pub struct CircleRepo {}

impl CircleRepo {
    // Circle SQL methods would go here when migrated from MongoDB to Postgres.
    // Currently CircleService uses MongoDB. Placeholder for future migration.
}
