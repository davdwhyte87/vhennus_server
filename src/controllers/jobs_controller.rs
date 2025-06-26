use actix_web::{get, web, HttpResponse};
use actix_web::web::{Data, ReqData};
use bigdecimal::{BigDecimal, FromPrimitive};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use sqlx::PgPool;
use crate::CONFIG;
use crate::models::live_rate_resp::LiveRateResponse;
use crate::models::profile::Profile;
use crate::models::response::GenericResp;
use crate::req_models::requests::UpdateProfileReq;
use crate::services::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};
use crate::services::email_service::EmailService;
use crate::services::post_service::PostService;
use crate::services::profile_service::{MiniProfile, ProfileService};
use crate::services::system_service::SystemService;
use crate::services::user_service::UserService;
use crate::utils::auth::Claims;


#[derive(Debug)]
struct Msg{
    pub title: String,
    pub description: String,
}

#[get("/comments_notify")]
pub async fn comments_notify(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let profiles = match PostService::get_last_1hr_comments(&pool).await{
        Ok(p) => p,
        Err(err)=>{
            log::error!("Error getting comments and post owners from past hour for notify {}", err);
            return HttpResponse::InternalServerError().json(respData);
        }
    };
    let message = Msg {title:String::from("Your post is getting attention 🎉"),
        description:String::from("You've received new comments on your post. Join the conversation!"),
    };
    for profile in profiles {

        if profile.token.is_some(){
            // send
            let payload = FcmMessage{
                message: MessagePayload {
                    token: profile.token.unwrap_or_default() ,
                    notification: Notification {
                        title: message.title.clone(),
                        body: message.description.clone()
                    },
                    data: None,
                },
            };
            send_app_notification(payload).await;
        }
    }
    return HttpResponse::Ok().json({})
}

#[get("/morning_notify")]
pub async fn morning_notify_job(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

  
    let profiles = match ProfileService::get_all(&pool).await {
        Ok(p) => p,
        Err(err) => {
            log::error!("error getting all users  {}", err.to_string());
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    #[derive(Debug)]
    struct Msg {
        title: String,
        description: String,
    }

    let messages = vec![
        Msg { title: "Everyone is a creator".into(), description: "Make 500 VEC everytime you post!".into() },
        Msg { title: "Earn over 5,000 naira daily".into(), description: "Make 10 VEC every minute you spend on the app!".into() },
        Msg { title: "Lets stack those coins".into(), description: "Make 500 VEC everytime you post!".into() },
    ];

    let mut rng = thread_rng();
    if let Some(message) = messages.choose(&mut rng) {
        for profile in profiles {
            if let Some(token) = profile.app_f_token {
                let payload = FcmMessage {
                    message: MessagePayload {
                        token,
                        notification: Notification {
                            title: message.title.clone(),
                            body: message.description.clone(),
                        },
                        data: None,
                    },
                };
                send_app_notification(payload).await;
            }
        }
    }
    return HttpResponse::Ok().json(respData)
}

#[get("/get_exchange_rate")]
pub async fn get_exchange_rate_job(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    let url = format!(
        "https://api.exchangerate.host/live?access_key={}",
        CONFIG.exchange_rate_api_key
    );
    println!("{}", url);
    let resp = match reqwest::get(&url).await{
        Ok(resp) => resp,
        Err(err) => {
            log::error!("{}", err);
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    let body = match resp.json::<LiveRateResponse>().await{
        Ok(body) => body,
        Err(e)=>{
            log::error!("{}",e);
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    // save to database
    let mut system_data = match SystemService::get_system_data(&pool).await{
        Ok(system_data) => {
            match system_data{
                Some(system_data) => system_data,
                None=>{
                   return HttpResponse::InternalServerError().json(respData)
                }
            }
        },
        Err(err)=>{
            log::error!("{}", err);
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    system_data.ngn = match body.quotes.get("USDNGN"){
        Some(ngn) => {
            BigDecimal::from_f64(*ngn).unwrap_or_default()
        },
        None=>{
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    match SystemService::update_system_data(&pool, system_data).await{
        Ok(s)=>{},
        Err(err)=>{
            log::error!("{}", err);

        }
    };
    println!("power {:?}", body);
    return HttpResponse::Ok().json(respData)
}

#[get("/referral_reminder")]
pub async fn referral_reminder(
    pool:Data<PgPool>,
    req: Result<web::Json<UpdateProfileReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<Vec<MiniProfile>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
    // get all users
    let users = match UserService::get_all(&pool).await{
        Ok(users) => users,
        Err(err)=>{
            respData.message = err.to_string();
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    for user in users{
        actix_rt::spawn(async move {
            if user.email.is_some(){
                let email_service = &EmailService::new();
                match EmailService::send_ref_reminder_email(email_service,user.email.unwrap_or_default()).await {
                    Ok(_) => {},
                    Err(err)=>{
                        log::error!("email error {}", err);
                    }
                };
            }
        });
    }
    respData.message = "Ok".to_string();
    return HttpResponse::Ok().json(respData)
}