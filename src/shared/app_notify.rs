use std::{env, error::Error, fs};
use std::path::PathBuf;
use actix_web::{web, Responder};
use awc::Client;
use dotenv::dotenv;
use gcp_auth::{CustomServiceAccount, TokenProvider};
use google_authenticator::GoogleAuthenticator;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};




/// Firebase project that owns the registration tokens. Override with
/// FCM_PROJECT_ID in .env — it MUST match the VITE_FIREBASE_PROJECT_ID
/// the web app was built with, otherwise FCM rejects every token.
/// Defaults preserve the old behaviour (dev project, prod project).
fn fcm_project_id() -> String {
    if let Ok(project) = env::var("FCM_PROJECT_ID") {
        let project = project.trim().to_owned();
        if !project.is_empty() {
            return project;
        }
    }
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".to_owned());
    if app_env == "prod" {
        "vhennus-916a0".to_owned()
    } else {
        "vhennus-dev".to_owned()
    }
}

fn fcm_send_url() -> String {
    format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send",
        fcm_project_id()
    )
}

#[derive(Serialize, Deserialize)]
pub struct FcmMessage {
    pub message: MessagePayload,
}

#[derive(Serialize, Deserialize)]
pub struct MessagePayload {
    pub token: String,
    pub notification: Notification,
    pub data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
}

pub async fn send_app_notification(payload: FcmMessage) -> Result<(), Box<dyn Error>> {
    log::debug!("starting token access ....");
    let access_token = match get_fcm_access_token().await {
        Ok(token) => token,
        Err(err) => return {
            log::error!("error getting access token {}", err);
            Err(err.into())
        },
    };

    log::debug!("got access token");
    // let payload = FcmMessage {
    //     message: MessagePayload {
    //         token: "cHYb-3M1QJ--sWZ2tc1MmX:APA91bGcWnvIKOI4ZTK9gmX0rUJpMOqd5eqjuflABd-VKgITlzD6MjpOC7NquCUaJrecG4aRTqGoZcSNM-j9g6n06SSF1yWP9yBSp3mmIJa0QsiWBG9aTWA".to_string(),
    //         notification: Notification {
    //             title: "Hello!".to_string(),
    //             body: "This is a test notification.".to_string(),
    //         },
    //         data: None,
    //     },
    // };

    let serialized_payload = serde_json::to_string(&payload)?;
    let client = Client::default();
    let fcm_url = fcm_send_url();
    log::info!("sending fcm push to project {} (token {}…)", fcm_project_id(), &payload.message.token[..payload.message.token.len().min(24)]);
    let mut response = client
        .post(&fcm_url)
        .insert_header(("Authorization", format!("Bearer {}", access_token))) // Correct way to add Bearer token
        .insert_header(("Content-Type", "application/json")) // Required for FCM
        .send_body(serialized_payload) // Send the serialized struct as JSON
        .await?;
    let status = response.status();
    if status.is_success() {
        log::debug!("fcm api response successful");
        Ok(())
    } else {
        let body = response.body().await.unwrap_or_default();
        let body_text = String::from_utf8_lossy(&body);
        log::error!(
            "error sending notification: status {} body {}",
            status,
            body_text
        );
        Err(format!("FCM send failed: status {} body {}", status, body_text).into())
    }
}


#[derive(Deserialize)]
struct FirebaseCredentials {
    client_email: String,
    private_key: String,
    token_uri: String,
}





async fn get_fcm_access_token() -> Result<String, Box<dyn std::error::Error>> {
    // Absolute path preferred: the binary's CWD varies by deployment
    // (Docker WORKDIR, systemd, deploy.sh build dir), and a relative
    // miss means every push fails with "file not found".
    let credentials_path = match env::var("FCM_SERVICE_ACCOUNT") {
        Ok(p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => PathBuf::from("service-account.json"),
    };
    if !credentials_path.exists() {
        return Err(format!(
            "service account file not found at {:?} (set FCM_SERVICE_ACCOUNT or place service-account.json in the working dir {:?})",
            credentials_path,
            std::env::current_dir().unwrap_or_default()
        )
        .into());
    }
    let service_account = CustomServiceAccount::from_file(credentials_path)?;
    let scopes = &["https://www.googleapis.com/auth/firebase.messaging"];
    let token = service_account.token(scopes).await?;
    Ok(token.as_str().to_string())
}