use std::error::Error;

use sqlx::PgPool;

use crate::notifications::models::NotificationItem;

pub struct NotificationRepo {}

impl NotificationRepo {
    pub async fn insert(
        pool: &PgPool,
        id: &str,
        user_name: &str,
        kind: &str,
        title: &str,
        body: &str,
        link: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "INSERT INTO notifications (id, user_name, kind, title, body, link)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(id)
        .bind(user_name)
        .bind(kind)
        .bind(title)
        .bind(body)
        .bind(link)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list(
        pool: &PgPool,
        user_name: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationItem>, Box<dyn Error>> {
        let rows = sqlx::query_as::<_, NotificationItem>(
            "SELECT id, user_name, kind, title, body, link, is_read, created_at
             FROM notifications WHERE user_name = $1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_name)
        .bind(limit.clamp(1, 50))
        .bind(offset.max(0))
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn unread_count(pool: &PgPool, user_name: &str) -> Result<i64, Box<dyn Error>> {
        #[derive(sqlx::FromRow)]
        struct Count {
            count: Option<i64>,
        }
        let row: Count = sqlx::query_as::<_, Count>(
            "SELECT COUNT(*) AS count FROM notifications WHERE user_name = $1 AND is_read = FALSE",
        )
        .bind(user_name)
        .fetch_one(pool)
        .await?;
        Ok(row.count.unwrap_or(0))
    }

    pub async fn mark_read(
        pool: &PgPool,
        user_name: &str,
        ids: Option<&[String]>,
    ) -> Result<(), Box<dyn Error>> {
        match ids {
            Some(list) if !list.is_empty() => {
                sqlx::query(
                    "UPDATE notifications SET is_read = TRUE
                     WHERE user_name = $1 AND id = ANY($2)",
                )
                .bind(user_name)
                .bind(list)
                .execute(pool)
                .await?;
            }
            _ => {
                sqlx::query(
                    "UPDATE notifications SET is_read = TRUE WHERE user_name = $1",
                )
                .bind(user_name)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
}
