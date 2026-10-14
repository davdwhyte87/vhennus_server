use std::collections::HashMap;
use std::error::Error;

use sqlx::PgPool;

use crate::notifications::models::{BroadcastResp, NotificationItem, PushStatusResp, UnreadCountResp};
use crate::notifications::repository::NotificationRepo;
use crate::profile::service::ProfileService;
use crate::shared::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};

pub struct NotificationService {}

impl NotificationService {
    pub async fn list(
        pool: &PgPool,
        user_name: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationItem>, Box<dyn Error>> {
        NotificationRepo::list(pool, user_name, limit, offset).await
    }

    pub async fn unread_count(
        pool: &PgPool,
        user_name: &str,
    ) -> Result<UnreadCountResp, Box<dyn Error>> {
        Ok(UnreadCountResp {
            count: NotificationRepo::unread_count(pool, user_name).await?,
        })
    }

    pub async fn mark_read(
        pool: &PgPool,
        user_name: &str,
        ids: Option<&[String]>,
    ) -> Result<(), Box<dyn Error>> {
        NotificationRepo::mark_read(pool, user_name, ids).await
    }

    pub async fn save_token(        pool: &PgPool,
        user_name: &str,
        token: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut profile = ProfileService::get_profile(pool, user_name.to_owned()).await?;
        profile.app_f_token = Some(token.to_owned());
        ProfileService::update_profile(pool, profile).await?;
        Ok(())
    }

    /// Admin visibility: how many users exist vs. how many devices can
    /// actually receive a push right now.
    pub async fn push_status(pool: &PgPool) -> Result<PushStatusResp, Box<dyn Error>> {
        let (total_users, users_with_tokens) = NotificationRepo::push_reach(pool).await?;
        Ok(PushStatusResp {
            total_users,
            users_with_tokens,
        })
    }

    /// Admin mass notification: one inbox row per user plus best-effort
    /// FCM push to every registered token. Returns delivery counts.
    pub async fn broadcast(
        pool: &PgPool,
        title: &str,
        body: &str,
        link: Option<&str>,
    ) -> Result<BroadcastResp, Box<dyn Error>> {
        let title = title.trim().to_string();
        let body = body.trim().to_string();
        if title.is_empty() || body.is_empty() {
            return Err("Title and body are required".into());
        }
        if title.len() > 140 || body.len() > 500 {
            return Err("Title (140) or body (500) too long".into());
        }
        let link: Option<String> = link
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty());

        let profiles = ProfileService::get_all(pool).await?;
        let mut users_notified: i64 = 0;
        let mut pushes_sent: i64 = 0;

        for profile in &profiles {
            let id = uuid::Uuid::new_v4().to_string();
            if NotificationRepo::insert(
                pool,
                &id,
                &profile.user_name,
                "broadcast",
                &title,
                &body,
                link.as_deref(),
            )
            .await
            .is_ok()
            {
                users_notified += 1;
            }

            if let Some(token) = profile.app_f_token.clone() {
                let mut data = HashMap::new();
                data.insert("kind".to_string(), "broadcast".to_string());
                if let Some(l) = &link {
                    data.insert("link".to_string(), l.clone());
                }
                let payload = FcmMessage {
                    message: MessagePayload {
                        token,
                        notification: Notification {
                            title: title.clone(),
                            body: body.clone(),
                        },
                        data: Some(data),
                    },
                };
                if send_app_notification(payload).await.is_ok() {
                    pushes_sent += 1;
                }
            }
        }

        Ok(BroadcastResp {
            users_notified,
            pushes_sent,
        })
    }

    /// Store an inbox row and best-effort push it via FCM when the
    /// recipient has a registered device/web token. Never fails the caller.
    pub async fn push(
        pool: &PgPool,
        recipient: &str,
        kind: &str,
        title: &str,
        body: &str,
        link: Option<&str>,
    ) {
        let id = uuid::Uuid::new_v4().to_string();
        if let Err(err) = NotificationRepo::insert(pool, &id, recipient, kind, title, body, link).await
        {
            log::error!("error storing notification {}", err);
            return;
        }

        let token: Option<String> = match ProfileService::get_profile(pool, recipient.to_owned()).await
        {
            Ok(p) => p.app_f_token,
            Err(err) => {
                log::error!("error getting profile for push {}", err);
                None
            }
        };

        if let Some(token) = token {
            let mut data = HashMap::new();
            data.insert("kind".to_string(), kind.to_string());
            if let Some(l) = link {
                data.insert("link".to_string(), l.to_string());
            }
            let payload = FcmMessage {
                message: MessagePayload {
                    token,
                    notification: Notification {
                        title: title.to_string(),
                        body: body.to_string(),
                    },
                    data: Some(data),
                },
            };
            if let Err(err) = send_app_notification(payload).await {
                log::error!("error sending push notification {}", err);
            }
        }
    }
}
