use std::sync::Arc;
use std::{env, error, fmt};

use std::fs::File;
use std::io::BufReader;
use actix_web::error::JsonPayloadError;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web::web::{resource, route, service, Data, JsonConfig, ServiceConfig};
use awc::Client;

mod controllers;
use controllers::buy_order_controller::seller_confirmed;
// use controllers::trivia_game_controller::{self, get_todays_game};
use controllers::{
    buy_order_controller, chats_controller, order_message_controller, payment_method_controller, post_controller, profile_controller, sell_order_controller, system_controller, user_controller, wallet_controller
};
mod models;
use dashmap::DashMap;
use dotenv::dotenv;
use get_if_addrs::get_if_addrs;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use r2d2_mongodb::mongodb::Error::IoError;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use models::{response, user};
mod database;
use database::db::db;
mod services;
use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use services::chat_session_service::UserConnections;
use services::{chat_session_service, user_service};
use crate::controllers::download_controller::download_apk;
use crate::controllers::jobs_controller;
use crate::models::user::User;
use crate::services::daily_post_job_service::{ get_exchange_rate_job, start_jobs};
use crate::services::jobs_service::AppScheduler;
use crate::services::mongo_service::MongoService;
mod utils;
mod req_models;
mod middlewares;



#[get("/hello")]
async fn index() -> impl Responder {
    "Hello, Bread!"
}
// #[get("/db_test")]
// async fn db_test(pool: web::Data<PgPool>)-> impl Responder {
//     let conn = &mut pool.get().expect("Couldn't get DB connection");
//     match users.load::<User>(conn) {
//         Ok(users_list) => HttpResponse::Ok().json(users_list),
//         Err(_) => HttpResponse::InternalServerError().finish(),
//     }
// }
// #[get("/{name}")]
// async fn hello(name: web::Path<String>) -> impl Responder {
//     format!("Hello {}!", &name)
// }


// Initialize the database pool
// fn init_db_pool() -> DbPool {
//     dotenv().ok();
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let manager = ConnectionManager::<PgConnection>::new(database_url);
//     r2d2::Pool::builder()
//         .build(manager)
//         .expect("Failed to create pool")
// }

async fn init_db_pool_x()-> PgPool{
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    //sqlx::migrate!().run(&pool).await.expect("Failed to run migration");
    return pool;
}

// fn load_rustls_config() -> ServerConfig {
//     let cert_file = &mut BufReader::new(File::open("cert.pem").expect("Cannot open certificate file"));
//     let key_file = &mut BufReader::new(File::open("key.pem").expect("Cannot open key file"));
// 
//     let cert_chain = rustls_pemfile::certs(cert_file)
//         .expect("Failed to read certificate")
//         .into_iter()
//         .map(Certificate)
//         .collect();
// 
//     let mut keys = rustls_pemfile::pkcs8_private_keys(key_file)
//         .expect("Failed to read private key");
// 
//     ServerConfig::builder()
//         .with_safe_defaults()
//         .with_no_client_auth()
//         .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))
//         .expect("Failed to create TLS server config")
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("Starting server..");

    dotenv().ok();

    env::set_var("RUST_BACKTRACE", "full");

    let app_env = match env::var("APP_ENV"){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("error getting mongo url var {}", err.to_string());
            panic!();
        }
    };

    let port = match env::var("PORT"){
        Ok(data)=>{
           data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "8000".to_string();
            panic!()
        }
    };

    // get computer ip
    let ifaces = get_if_addrs().expect("Failed to get network interfaces");

    // Filter for the first non-loopback IPv4 address
    let ip_address = ifaces.iter()
        .filter(|iface| iface.ip().is_ipv4() && !iface.is_loopback())
        .map(|iface| iface.ip())
        .next()
        .expect("No valid IPv4 address found");

    let mut  address =format!("{}:{}","0.0.0.0", port);
    if app_env == "test" || app_env=="prod"{
        address =format!("{}:{}",ip_address, port);
    }
    info!("Starting server on {}", address);

    // hashmap for holding websocket connections for chat
    let user_connections: UserConnections = Arc::new(DashMap::new());
    //let pool = init_db_pool();
    let pool = init_db_pool_x().await;
    
    // start daily post job
    //start_jobs(pool.clone()).await;


    if (app_env == "test" || app_env=="prod"){
        let ssl_config = load_ssl_config(
            "/etc/letsencrypt/live/bend.vhennus.com/fullchain.pem",
            "/etc/letsencrypt/live/bend.vhennus.com/privkey.pem"
        ).expect("Failed to load ssl config");
        HttpServer::new(move|| {

            App::new()
                .app_data(Data::new(pool.clone()))
                .app_data(web::Data::new(user_connections.clone())) // pass data to routes if needed
                .configure(configure_services)
        })

            .bind_rustls(address, ssl_config)?
            .run()
            .await
    }else if app_env == "local" {
        HttpServer::new(move|| {

            App::new()
                .app_data(Data::new(pool.clone()))
                .app_data(web::Data::new(user_connections.clone())) // pass data to routes if needed
                .configure(configure_services)
        })
            .bind(address)?
            .run()
            .await
    }else {
        panic!("Set Environment config");
    }
}



fn configure_services(cfg: &mut ServiceConfig) {
    cfg
        .service(
            web::scope("api/v1/auth")
                .service(user_controller::say_hello)
                .wrap(middlewares::auth_middleware::AuthM)
                .service(wallet_controller::buy_coin)
                .service(wallet_controller::get_wallet)
                .service(
                    web::scope("post")
                        .service(post_controller::create_post)
                        .service(post_controller::create_comment)
                        .service(post_controller::get_all_posts)
                        .service(post_controller::get_my_posts)
                        .service(post_controller::get_single_posts)
                        .service(post_controller::like_post)
                        .service(post_controller::get_users_posts),
                )
                .service(
                    web::scope("profile")
                        .service(profile_controller::update_profile)
                        .service(profile_controller::get_profile)
                        .service(profile_controller::get_user_profile)
                        .service(profile_controller::get_friends)
                        .service(profile_controller::search)
                        .service(profile_controller::get_friend_suggestion)
                        .service(profile_controller::add_wallet)
                        .service(profile_controller::activate_earnings)
                        .service(profile_controller::cashout_earnings)
                        .service(profile_controller::post_earnings)
                )
                .service(
                    web::scope("user")
                        .service(user_controller::accept_friend_request)
                        .service(user_controller::reject_friend_request)
                        .service(user_controller::send_friend_request)
                        .service(user_controller::get_my_friend_request)
                        .service(user_controller::delete_profile)
                )
                .service(
                    web::scope("chat")
                        .service(chats_controller::create_chat)
                        .service(chats_controller::get_by_pair)
                        .service(chats_controller::get_my_chat_pairs)
                        .service(chats_controller::find_chat_pair)
                        .route("/ws", web::get().to(chats_controller::we_chat_connect)),
                )
            ,
        )
        .service(index)
        .route("/ws", web::get().to(chat_session_service::ws_chat))
        .service(user_controller::create_account)
        .service(user_controller::login)
        .service(user_controller::confirm_account)
        .service(user_controller::resend_code)
        .service(system_controller::get_system_data)
        .service(download_apk)
        .service(user_controller::get_reset_password_code)
        .service(user_controller::change_password)
        .service(
            web::scope("cron_jobs")
                .service(jobs_controller::get_exchange_rate_job)
                .service(jobs_controller::morning_notify_job)
                .service(jobs_controller::comments_notify)
        )
    ;
}



fn load_ssl_config(cert_path: &str, key_path: &str) ->Result<ServerConfig, Box<dyn std::error::Error>> {
    // Load certificate
    let cert_file = File::open(cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = certs(&mut cert_reader)
        .map(|certs| certs.into_iter().map(Certificate).collect())
        .expect("Failed to load certificate chain");

    // Load private key
    let key_file = File::open(key_path)?;
    let mut key_reader = BufReader::new(key_file);
    let mut keys = pkcs8_private_keys(&mut key_reader)
        .expect("Failed to load private keys");

    if keys.is_empty() {
        panic!("No private keys found");
    }

    let key = PrivateKey(keys.remove(0));

    // Configure TLS
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("Failed to create ServerConfig");

    Ok(config)
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: String,
    pub database_url: String,
    pub email:String,
    pub email_password:String,
    pub exchange_rate_api_key:String,
    pub blockchain_ip:String,
    pub earnings_wallet:String,
    pub earnings_wallet_password:String,
}


pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv().ok();

    let port = match env::var("PORT"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "8000".to_string();
            return panic!()
        }
    };

    let database_url = match env::var("DATABASE_URL"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "8000".to_string();
            panic!()
        }
    };

    let email = match env::var("EMAIL"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "8000".to_string();
            panic!()
        }
    };
    let email_password = match env::var("EMAIL_PASSWORD"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "8000".to_string();
            panic!()
        }
    };
    let exchange_rate_api_key = match env::var("EXCHANGE_API_KEY"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "".to_string();
            panic!()
        }
    };    
    let earnings_wallet_password = match env::var("EARNINGS_WALLET_PASSWORD"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "".to_string();
            panic!()
        }
    };
    let earnings_wallet = match env::var("EARNINGS_WALLET"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "".to_string();
            panic!()
        }
    };
    let blockchain_ip = match env::var("BLOCKCHAIN_IP"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "".to_string();
            panic!()
        }
    };
    Config{
        port: port,
        email:email,
        database_url:database_url,
        email_password:email_password,
        exchange_rate_api_key: exchange_rate_api_key,
        earnings_wallet,
        earnings_wallet_password,
        blockchain_ip
    }
});
