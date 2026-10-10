use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, http, options, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::web::{Data, ServiceConfig};
use dashmap::DashMap;
use dotenv::dotenv;
use log::{debug, error, info};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

mod chat;
mod feed;
mod groups;
mod jobs;
mod membership;
mod middlewares;
mod orders;
mod profile;
mod shared;
mod system;
mod trivia;
mod user;
mod wallet;

use crate::chat::service::UserConnections;
use crate::chat::service::ChatSessions;
use crate::groups::models::{GroupPresence, GroupSessions};
use crate::shared::config::CONFIG;

#[get("/hello")]
async fn index(req: HttpRequest) -> impl Responder {
    if let Some(cookie)= req.cookie("clickId"){
        return HttpResponse::Ok().body(format!("hello bread and cookie clickId = {}", cookie.value()))
    }
    HttpResponse::Ok().body("Hello bread!")
}

async fn init_db_pool_x()-> PgPool{
    dotenv().ok();
    let database_url = CONFIG.database_url.to_owned();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    //sqlx::migrate!().run(&pool).await.expect("Failed to run migration");
    return pool;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("Starting server..");

    dotenv().ok();

    //env::set_var("RUST_BACKTRACE", "full");
    let config = &*CONFIG;

    let port: u16 = CONFIG.port.to_owned().parse().ok()  // Option<u16>
        .unwrap();
    let address = ("0.0.0.0", port);
    info!("Starting server on {:?}", address);
    debug!("Starting server on {:?}", address);
    debug!("App env {:?}", CONFIG.app_env);
    // hashmap for holding websocket connections for chat
    let user_connections: UserConnections = Arc::new(DashMap::new());
    let chat_sessions: ChatSessions = Arc::new(Mutex::new(HashMap::new()));
    let group_sessions: GroupSessions = Arc::new(DashMap::new());
    let group_presence: GroupPresence = Arc::new(DashMap::new());
    //let pool = init_db_pool();
    let pool = init_db_pool_x().await;

    // start daily post job
    //start_jobs(pool.clone()).await;



    if(config.app_env == "test" ||config.app_env ==  "local"){
        HttpServer::new(move|| {
            let cors = Cors::default()
                .allowed_origin("http://localhost:5173")
                .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                    http::header::HeaderName::from_static("x-requested-with"),
                ])
                .supports_credentials()
                .max_age(3600);

            App::new()
                .wrap(Logger::default()) // This will log all requests
                .wrap(cors)
                .app_data(Data::new(pool.clone()))
                .app_data(web::Data::new(user_connections.clone()))
                .app_data(web::Data::new(chat_sessions.clone()))
                .app_data(web::Data::new(group_sessions.clone()))
                .app_data(web::Data::new(group_presence.clone()))
                
                .configure(configure_services)
        })
            .bind(address)?
            .run()
            .await   
    }else {
        HttpServer::new(move|| {
            let cors_prod =  Cors::default()
                .allowed_origin("https://www.vhennus.org")
                .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                    http::header::HeaderName::from_static("x-requested-with"),
                ])
                .supports_credentials()
                .max_age(3600);
            App::new()
                .wrap(cors_prod)
                .app_data(Data::new(pool.clone()))
                .app_data(web::Data::new(user_connections.clone()))
                .app_data(web::Data::new(chat_sessions.clone()))
                .app_data(web::Data::new(group_sessions.clone()))
                .app_data(web::Data::new(group_presence.clone()))
                .configure(configure_services)
        })
            .bind(address)?
            .run()
            .await
    }

}


#[options("/api/v1/auth/{tail:.*}")]
async fn options_handler() -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS"))
        .append_header(("Access-Control-Allow-Headers", "Content-Type, Authorization, x-requested-with"))
        .append_header(("Access-Control-Max-Age", "3600"))
        .finish()
}

fn configure_services(cfg: &mut ServiceConfig) {

    cfg
        .service(options_handler)
        .service(
            web::scope("api/v1/auth")
                .service(user::controller::say_hello)
                .wrap(middlewares::auth_middleware::AuthM)
                .service(wallet::controller::buy_coin)
                .service(wallet::controller::get_wallet)
                .service(
                    web::scope("post")
                        .service(feed::controller::create_post)
                        .service(feed::controller::create_comment)
                        .service(feed::controller::get_all_posts)
                        .service(feed::controller::get_my_posts)
                        .service(feed::controller::get_single_posts)
                        .service(feed::controller::like_post)
                        .service(feed::controller::get_users_posts),
                )
                .service(
                    web::scope("profile")
                        .service(profile::controller::update_profile)
                        .service(profile::controller::get_profile)
                        .service(profile::controller::get_user_profile)
                        .service(profile::controller::get_friends)
                        .service(profile::controller::search)
                        .service(profile::controller::get_friend_suggestion)
                        .service(profile::controller::add_wallet)
                        .service(profile::controller::activate_earnings)
                        .service(profile::controller::cashout_earnings)
                        .service(profile::controller::post_earnings)
                )
                .service(
                    web::scope("user")
                        .service(user::controller::accept_friend_request)
                        .service(user::controller::reject_friend_request)
                        .service(user::controller::unfriend)
                        .service(user::controller::send_friend_request)
                        .service(user::controller::get_my_friend_request)
                        .service(user::controller::delete_profile)
                )
                .service(
                    web::scope("group")
                        // Groups: one group = one feed (no rooms)
                        .service(groups::controller_v2::create_group_v2)
                        .service(groups::controller_v2::update_group_v2)
                        .service(groups::controller_v2::my_groups_v2)
                        .service(groups::controller_v2::search_groups_v2)
                        .service(groups::controller_v2::unread_v2)
                        .service(groups::controller_v2::list_categories_v2)
                        .service(groups::controller_v2::invite_preview_v2)
                        .service(groups::controller_v2::get_group_v2)
                        .service(groups::controller_v2::group_members_v2)
                        .service(groups::controller_v2::remove_member_v2)
                        .service(groups::controller_v2::join_group_v2)
                        .service(groups::controller_v2::list_requests_v2)
                        .service(groups::controller_v2::respond_request_v2)
                        .service(groups::controller_v2::add_topic_v2)
                        .service(groups::controller_v2::close_topic_v2)
                        .service(groups::controller_v2::group_messages_v2)
                        .service(groups::controller_v2::mark_read_v2)
                        .service(groups::controller_v2::admin_list_groups_v2)
                        .service(groups::controller_v2::admin_get_group_v2)
                        .service(groups::controller_v2::admin_add_category_v2)
                        .service(groups::controller_v2::admin_delete_category_v2)
                )
                .service(
                    web::scope("membership")
                        .service(membership::controller::get_questions)
                        .service(membership::controller::create_application)
                        .service(membership::controller::my_application)
                        .service(membership::controller::membership_status)
                        .service(membership::controller::submit_answers),
                )
                .service(
                    web::scope("admin/membership")
                        .service(membership::controller::list_applications)
                        .service(membership::controller::get_application)
                        .service(membership::controller::approve_application)
                        .service(membership::controller::reject_application)
                        .service(membership::controller::review_application)
                        .service(membership::controller::admin_list_questions)
                        .service(membership::controller::admin_create_question)
                        .service(membership::controller::admin_update_question)
                        .service(membership::controller::admin_delete_question)
                        .service(membership::controller::admin_stats)
                        .service(membership::controller::admin_list_users)
                        .service(membership::controller::admin_get_settings)
                        .service(membership::controller::admin_update_settings),
                )
                .service(
                    web::scope("chat")
                        .service(chat::controller::create_chat)
                        .service(chat::controller::get_by_pair)
                        .service(chat::controller::get_chats)
                        .service(chat::controller::get_my_chat_pairs)
                        .service(chat::controller::find_chat_pair)
                        .service(chat::controller::get_unread)
                        .service(chat::controller::mark_read)
                )
            ,
        )
        .service(index)
        .route("/ws", web::get().to(chat::service::ws_chat))
        .route("/chat/ws", web::get().to(chat::controller::wsocket_chat_connect) )
        .route("/group/ws", web::get().to(groups::controller_v2::ws_group_v2_connect))
        .service(user::controller::create_account)
        .service(user::controller::login)
        .service(user::controller::confirm_account)
        .service(user::controller::resend_code)
        .service(system::controller::get_system_data)
        .service(jobs::controller::download_apk)
        .service(user::controller::get_reset_password_code)
        .service(user::controller::change_password)
        .service(
            web::scope("cron_jobs")
                .service(jobs::controller::get_exchange_rate_job)
                .service(jobs::controller::morning_notify_job)
                .service(jobs::controller::comments_notify)
                .service(jobs::controller::referral_reminder)
                .service(jobs::controller::event_reminder)
        )
    ;
}