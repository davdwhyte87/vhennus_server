use sqlx::PgPool;

use crate::search_tags::models::{normalize_name, SearchTag};

pub struct SearchTagsRepo;

impl SearchTagsRepo {
    pub async fn list_all(pool: &PgPool, kind: Option<String>) -> Result<Vec<SearchTag>, sqlx::Error> {
        if let Some(kind) = kind {
            sqlx::query_as::<_, SearchTag>(
                "SELECT id, name, kind, created_at, updated_at FROM search_tags WHERE kind = $1 ORDER BY name ASC",
            )
            .bind(kind)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, SearchTag>(
                "SELECT id, name, kind, created_at, updated_at FROM search_tags ORDER BY kind ASC, name ASC",
            )
            .fetch_all(pool)
            .await
        }
    }

    pub async fn get_by_ids(pool: &PgPool, ids: &[String]) -> Result<Vec<SearchTag>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        sqlx::query_as::<_, SearchTag>(
            "SELECT id, name, kind, created_at, updated_at FROM search_tags WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &PgPool, id: &str, name: &str, kind: &str) -> Result<SearchTag, sqlx::Error> {
        sqlx::query_as::<_, SearchTag>(
            "INSERT INTO search_tags(id, name, kind) VALUES ($1, $2, $3)
             ON CONFLICT (name, kind) DO NOTHING
             RETURNING id, name, kind, created_at, updated_at",
        )
        .bind(id)
        .bind(normalize_name(name))
        .bind(kind)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_name_kind(pool: &PgPool, name: &str, kind: &str) -> Result<Option<SearchTag>, sqlx::Error> {
        sqlx::query_as::<_, SearchTag>(
            "SELECT id, name, kind, created_at, updated_at FROM search_tags WHERE name = $1 AND kind = $2",
        )
        .bind(normalize_name(name))
        .bind(kind)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(pool: &PgPool, id: &str, name: Option<&str>, kind: Option<&str>) -> Result<Option<SearchTag>, sqlx::Error> {
        sqlx::query_as::<_, SearchTag>(
            "UPDATE search_tags SET
               name = COALESCE($2, name),
               kind = COALESCE($3, kind),
               updated_at = NOW()
             WHERE id = $1
             RETURNING id, name, kind, created_at, updated_at",
        )
        .bind(id)
        .bind(name.map(normalize_name))
        .bind(kind)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM search_tags WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn get_my_tag_ids(pool: &PgPool, user_name: &str) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT tag_id FROM profile_search_tags WHERE profile_user_name = $1",
        )
        .bind(user_name)
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn set_my_tags(pool: &PgPool, user_name: &str, tag_ids: &[String]) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM profile_search_tags WHERE profile_user_name = $1")
            .bind(user_name)
            .execute(&mut *tx)
            .await?;
        for tag_id in tag_ids {
            sqlx::query(
                "INSERT INTO profile_search_tags(profile_user_name, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(user_name)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn tags_for_users(
        pool: &PgPool,
        user_names: &[String],
    ) -> Result<Vec<(String, SearchTag)>, sqlx::Error> {
        if user_names.is_empty() {
            return Ok(vec![]);
        }
        #[derive(sqlx::FromRow)]
        struct Row {
            profile_user_name: String,
            id: String,
            name: String,
            kind: String,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT m.profile_user_name, t.id, t.name, t.kind, t.created_at, t.updated_at
             FROM profile_search_tags m
             JOIN search_tags t ON t.id = m.tag_id
             WHERE m.profile_user_name = ANY($1)",
        )
        .bind(user_names)
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.profile_user_name,
                    SearchTag {
                        id: r.id,
                        name: r.name,
                        kind: r.kind,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    },
                )
            })
            .collect())
    }
}
