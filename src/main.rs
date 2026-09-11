use std::sync::Arc;

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
mod middlewares;
mod orders;
mod profile;
mod shared;
mod system;
mod trivia;
mod user;
mod wallet;

use crate::chat::service::UserConnections;
use crate::groups::models::{RoomMembers, UserRoomSessions};
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
    let room_members: RoomMembers = Arc::new(DashMap::new());
    let user_room_sessions: UserRoomSessions = Arc::new(DashMap::new());
    //let pool = init_db_pool();
    let pool = init_db_pool_x().await;

    // start daily post job
    //start_jobs(pool.clone()).await;



    if(config.app_env == "test" ||config.app_env ==  "local"){
        HttpServer::new(move|| {
            let cors = Cors::default()
                .allowed_origin("http://localhost:5173")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
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
                .app_data(web::Data::new(room_members.clone()))
                .app_data(web::Data::new(user_room_sessions.clone()))
                
                .configure(configure_services)
        })
            .bind(address)?
            .run()
            .await   
    }else {
        HttpServer::new(move|| {
            let cors_prod =  Cors::default()
                .allowed_origin("https://www.vhennus.com")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
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
                .app_data(web::Data::new(room_members.clone()))
                .app_data(Data::new(user_room_sessions.clone()))
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
        .append_header(("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"))
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
                        .service(user::controller::send_friend_request)
                        .service(user::controller::get_my_friend_request)
                        .service(user::controller::delete_profile)
                )
                .service(
                    web::scope("group")
                        .service(groups::controller::create_group)
                        .service(groups::controller::create_room)
                        .service(groups::controller::join_room)
                        .service(groups::controller::join_room_with_code)
                        .service(groups::controller::generate_room_code)
                        .service(groups::controller::update_group)
                        .service(groups::controller::update_room)
                        .service(groups::controller::leave_room)
                        .service(groups::controller::get_my_groups)
                        .service(groups::controller::get_group)
                        .service(groups::controller::get_room)
                        .route("/ws_group", web::get().to(groups::controller::connect_to_rooms))
                )
                .service(
                    web::scope("chat")
                        .service(chat::controller::create_chat)
                        .service(chat::controller::get_by_pair)
                        .service(chat::controller::get_chats)
                        .service(chat::controller::get_my_chat_pairs)
                        .service(chat::controller::find_chat_pair)
                        .route("/ws", web::get().to(chat::controller::we_chat_connect)),
                )
            ,
        )
        .service(index)
        .route("/ws", web::get().to(chat::service::ws_chat))
        .route("/chat/ws", web::get().to(chat::controller::wsocket_chat_connect) )
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