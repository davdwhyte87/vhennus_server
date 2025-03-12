use std::{env, error::Error, fs};
use std::path::PathBuf;
use actix_web::{web, Responder};
use awc::Client;
use dotenv::dotenv;
use gcp_auth::{CustomServiceAccount, TokenProvider};
use google_authenticator::GoogleAuthenticator;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};




const FCM_URL: &str = "https://fcm.googleapis.com/v1/projects/vhennus-dev/messages:send";

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
    pub user_name: String
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
    let mut fcm_url = FCM_URL;
    let app_env = match env::var("APP_ENV"){
        Ok(data)=>{data},
        Err(err)=>{
            "local".to_owned()
        }
    };

    if app_env == "prod" {
        fcm_url =  "https://fcm.googleapis.com/v1/projects/vhennus-916a0/messages:send"
    }
    let response = client
        .post(fcm_url)
        .insert_header(("Authorization", format!("Bearer {}", access_token))) // Correct way to add Bearer token
        .insert_header(("Content-Type", "application/json")) // Required for FCM
        .send_body(serialized_payload) // Send the serialized struct as JSON
        .await?;
    if response.status().is_success() {
        log::debug!("fcm api response successful");
        Ok(())
    } else {
        log::error!("error sending notification: {}", response.status());
        Err(Box::from("FAILED"))
    }
}


#[derive(Deserialize)]
struct FirebaseCredentials {
    client_email: String,
    private_key: String,
    token_uri: String,
}





async fn get_fcm_access_token() -> Result<String, Box<dyn std::error::Error>> {
    let credentials_path = PathBuf::from("service-account.json");
    let service_account = CustomServiceAccount::from_file(credentials_path)?;
    let scopes = &["https://www.googleapis.com/auth/firebase.messaging"];
    let token = service_account.token(scopes).await?;
    Ok(token.as_str().to_string())
}

