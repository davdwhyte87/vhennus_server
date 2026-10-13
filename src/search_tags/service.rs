use sqlx::PgPool;
use uuid::Uuid;

use crate::search_tags::models::{
    normalize_kind, normalize_name, FriendSearchHit, MyTagsResp, SearchFriendsReq, SearchTag,
    MAX_INTERESTS_PER_USER, MAX_SKILLS_PER_USER, TAG_KIND_INTEREST, TAG_KIND_SKILL,
};
use crate::search_tags::repository::SearchTagsRepo;
use crate::shared::auth::Claims;
use crate::shared::error::AppError;

pub struct SearchTagsService;

impl SearchTagsService {
    fn ensure_admin(claim: &Claims) -> Result<(), AppError> {
        if claim.role != "ADMIN" {
            return Err(AppError::UnauthorizedError);
        }
        Ok(())
    }

    pub async fn list(pool: &PgPool, kind: Option<String>) -> Result<Vec<SearchTag>, AppError> {
        if let Some(ref k) = kind {
            if normalize_kind(k).is_none() {
                return Err(AppError::BadRequestError("kind must be 'skill' or 'interest'".to_string()));
            }
        }
        let normalized = kind.map(|k| normalize_kind(&k).unwrap_or(k));
        SearchTagsRepo::list_all(pool, normalized)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error listing tags: {}", e)))
    }

    pub async fn create(pool: &PgPool, claim: &Claims, name: String, kind: String) -> Result<SearchTag, AppError> {
        Self::ensure_admin(claim)?;
        let kind = normalize_kind(&kind)
            .ok_or_else(|| AppError::BadRequestError("kind must be 'skill' or 'interest'".to_string()))?;
        let clean = normalize_name(&name);
        if clean.is_empty() {
            return Err(AppError::BadRequestError("name is required".to_string()));
        }
        if clean.len() > 80 {
            return Err(AppError::BadRequestError("name is too long (max 80)".to_string()));
        }
        // Return existing row instead of erroring on duplicates.
        if let Ok(Some(existing)) = SearchTagsRepo::find_by_name_kind(pool, &clean, &kind).await {
            return Ok(existing);
        }
        let id = format!("{}-{}", kind, Uuid::new_v4());
        SearchTagsRepo::create(pool, &id, &clean, &kind)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error creating tag (maybe duplicate): {}", e)))
    }

    pub async fn update(
        pool: &PgPool,
        claim: &Claims,
        id: String,
        name: Option<String>,
        kind: Option<String>,
    ) -> Result<SearchTag, AppError> {
        Self::ensure_admin(claim)?;
        let kind_norm = match kind {
            Some(k) => Some(
                normalize_kind(&k)
                    .ok_or_else(|| AppError::BadRequestError("kind must be 'skill' or 'interest'".to_string()))?,
            ),
            None => None,
        };
        let name_norm = match name {
            Some(n) => {
                let c = normalize_name(&n);
                if c.is_empty() {
                    return Err(AppError::BadRequestError("name is required".to_string()));
                }
                Some(c)
            }
            None => None,
        };
        SearchTagsRepo::update(pool, &id, name_norm.as_deref(), kind_norm.as_deref())
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error updating tag: {}", e)))?
            .ok_or_else(|| AppError::NotFoundError("tag".to_string(), id))
    }

    pub async fn delete(pool: &PgPool, claim: &Claims, id: String) -> Result<(), AppError> {
        Self::ensure_admin(claim)?;
        let removed = SearchTagsRepo::delete(pool, &id)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error deleting tag: {}", e)))?;
        if !removed {
            return Err(AppError::NotFoundError("tag".to_string(), id));
        }
        Ok(())
    }

    pub async fn get_my_tags(pool: &PgPool, claim: &Claims) -> Result<MyTagsResp, AppError> {
        let ids = SearchTagsRepo::get_my_tag_ids(pool, &claim.user_name)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error getting tags: {}", e)))?;
        let tags = SearchTagsRepo::get_by_ids(pool, &ids)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error getting tags: {}", e)))?;
        Ok(MyTagsResp { tag_ids: ids, tags })
    }

    pub async fn set_my_tags(pool: &PgPool, claim: &Claims, tag_ids: Vec<String>) -> Result<MyTagsResp, AppError> {
        // Validate ids exist (ignore unknown silently? No — reject to catch stale clients).
        let tags = SearchTagsRepo::get_by_ids(pool, &tag_ids)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error validating tags: {}", e)))?;
        if tags.len() != tag_ids.len() {
            return Err(AppError::BadRequestError("One or more selected skills/interests are invalid".to_string()));
        }
        let skill_count = tags.iter().filter(|t| t.kind == TAG_KIND_SKILL).count();
        let interest_count = tags.iter().filter(|t| t.kind == TAG_KIND_INTEREST).count();
        if skill_count > MAX_SKILLS_PER_USER {
            return Err(AppError::BadRequestError(format!(
                "You can select at most {} skills",
                MAX_SKILLS_PER_USER
            )));
        }
        if interest_count > MAX_INTERESTS_PER_USER {
            return Err(AppError::BadRequestError(format!(
                "You can select at most {} interests",
                MAX_INTERESTS_PER_USER
            )));
        }
        SearchTagsRepo::set_my_tags(pool, &claim.user_name, &tag_ids)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error saving tags: {}", e)))?;
        Ok(MyTagsResp { tag_ids, tags })
    }

    /// Friend search: text query (name / username / bio) + ALL-of selected tags.
    pub async fn search_friends(
        pool: &PgPool,
        claim: &Claims,
        req: SearchFriendsReq,
    ) -> Result<Vec<FriendSearchHit>, AppError> {
        let limit = req.limit.unwrap_or(30).clamp(1, 50);
        let skill_ids = req.skill_ids.unwrap_or_default();
        let interest_ids = req.interest_ids.unwrap_or_default();
        let query = req.query.unwrap_or_default().trim().to_string();

        if query.is_empty() && skill_ids.is_empty() && interest_ids.is_empty() {
            return Ok(vec![]);
        }

        #[derive(sqlx::FromRow)]
        struct Hit {
            user_name: String,
            name: Option<String>,
            bio: Option<String>,
            image: Option<String>,
        }

        // Build dynamic SQL: text match AND EXISTS(tag) for every selected tag.
        let mut sql = String::from(
            "SELECT p.user_name, p.name, p.bio, p.image FROM profiles p WHERE p.user_name <> $1",
        );
        let mut idx: i32 = 2;
        if !query.is_empty() {
            sql.push_str(&format!(
                " AND (p.user_name ILIKE ${} OR p.name ILIKE ${} OR p.bio ILIKE ${})",
                idx, idx, idx
            ));
            idx += 1;
        }
        let mut tag_count = 0;
        for _ in skill_ids.iter().chain(interest_ids.iter()) {
            sql.push_str(&format!(
                " AND EXISTS (SELECT 1 FROM profile_search_tags m WHERE m.profile_user_name = p.user_name AND m.tag_id = ${})",
                idx
            ));
            idx += 1;
            tag_count += 1;
        }
        sql.push_str(&format!(" ORDER BY p.name ASC NULLS LAST, p.user_name ASC LIMIT ${}", idx));

        let like = format!("%{}%", query);
        let mut q = sqlx::query_as::<_, Hit>(&sql).bind(&claim.user_name);
        if !query.is_empty() {
            q = q.bind(&like);
        }
        for tag_id in skill_ids.iter().chain(interest_ids.iter()) {
            q = q.bind(tag_id);
        }
        let _ = tag_count;
        let hits: Vec<Hit> = q
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error searching users: {}", e)))?;

        let user_names: Vec<String> = hits.iter().map(|h| h.user_name.clone()).collect();
        let tagged = SearchTagsRepo::tags_for_users(pool, &user_names)
            .await
            .map_err(|e| AppError::BadRequestError(format!("Error loading tags: {}", e)))?;

        use std::collections::HashMap;
        let mut by_user: HashMap<String, Vec<SearchTag>> = HashMap::new();
        for (uname, tag) in tagged {
            by_user.entry(uname).or_default().push(tag);
        }

        Ok(hits
            .into_iter()
            .map(|h| {
                let tags = by_user.remove(&h.user_name).unwrap_or_default();
                let (skills, interests): (Vec<SearchTag>, Vec<SearchTag>) =
                    tags.into_iter().partition(|t| t.kind == TAG_KIND_SKILL);
                // Keep interests that are kind==interest (partition puts non-skill there, but be explicit)
                let interests: Vec<SearchTag> =
                    interests.into_iter().filter(|t| t.kind == TAG_KIND_INTEREST).collect();
                FriendSearchHit {
                    user_name: h.user_name,
                    name: h.name,
                    bio: h.bio,
                    image: h.image,
                    skills,
                    interests,
                }
            })
            .collect())
    }
}
