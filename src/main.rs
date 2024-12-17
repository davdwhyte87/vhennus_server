use std::{env, error, fmt};
use actix_web::error::JsonPayloadError;
use actix_web::{ get, web, App, Error, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web::web::{resource, route, service, Data, JsonConfig};


mod controllers;
use controllers::buy_order_controller::seller_confirmed;
use controllers::trivia_game_controller::{self, get_todays_game};
use controllers::{
    buy_order_controller, chats_controller, order_message_controller, payment_method_controller, post_controller, profile_controller, sell_order_controller, system_controller, user_controller, wallet_controller

};
mod models;
use dotenv::dotenv;
use get_if_addrs::get_if_addrs;
use log::{info, error, debug};
use models::{response, user};
mod database;
use database::db::db;
mod services;
use serde_json::json;
use services::{user_service};
use thiserror::Error;
use crate::services::mongo_service::MongoService;
mod utils;
mod req_models;
mod middlewares;





#[get("/hello")]
async fn index() -> impl Responder {
    "Hello, Bread!"
}

// #[get("/{name}")]
// async fn hello(name: web::Path<String>) -> impl Responder {
//     format!("Hello {}!", &name)
// }



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("Starting server..");

    dotenv().ok();

    env::set_var("RUST_BACKTRACE", "full");
    let db = MongoService::init().await;
    let db_data = Data::new(db);

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
    HttpServer::new(move|| {
        
        App::new()
            .app_data(db_data.clone())
            
            // USER CONTROLLERS

            .service(
                // all authenticated endpoints
                web::scope("api/v1/auth")
                
                    .service(user_controller::say_hello)
                    .wrap(middlewares::auth_middleware::AuthM)
                  

                    // runstats
                  
                    .service(wallet_controller::buy_coin)
                    .service(wallet_controller::get_wallet)
                    

                    // sell order

                    .service(
                        web::scope("sell_order")
                        .service(sell_order_controller::create_sell_order)
                        .service(sell_order_controller::get_my_sell_orders)
                        .service(sell_order_controller::get_single_sell_order)
                        .service(sell_order_controller::cancel_sell_order)
                        .service(sell_order_controller::update_sell_order)
                        .service(sell_order_controller::get_all_open_sell_orders)
                    )
                    .service(
                        web::scope("buy_order")
                        .service(buy_order_controller::create_buy_order)
                        .service(buy_order_controller::get_my_buy_orders)
                        .service(buy_order_controller::get_single_buy_order)
                        .service(buy_order_controller::buyer_confirmed)
                        .service(buy_order_controller::seller_confirmed)
                        .service(buy_order_controller::cancel_buy_order)
                    )
                    .service(
                        web::scope("payment_method")
                        .service(payment_method_controller::create_payment_method)
                        .service(payment_method_controller::delete_payment_method)
                        .service(payment_method_controller::get_my_payment_methods)
                    )
                    .service(
                        web::scope("order_message")
                        .service(order_message_controller::create_order_message)
                        .service(order_message_controller::get_order_message)
                    )
                    .service(
                        web::scope("post")
                        .service(post_controller::create_post)
                        .service(post_controller::create_comment)
                        .service(post_controller::get_all_posts)
                        .service(post_controller::get_single_posts)
                        .service(post_controller::like_post)
                    )

                    .service(
                        web::scope("trivia")
                        .service(trivia_game_controller::get_todays_game)  
                        .service(trivia_game_controller::play_game)
                    )
                    .service(
                        web::scope("profile")
                        .service(profile_controller::update_profile)
                        .service(profile_controller::get_profile)
                        .service(profile_controller::get_friends)
                        .service(profile_controller::search)
                    )
                    .service(
                        web::scope("user")
                        .service(user_controller::accept_friend_request)
                        .service(user_controller::send_friend_request)
                        .service(user_controller::get_my_friend_request)
                    )
                    .service(
                        web::scope("chat")
                        .service(chats_controller::create_chat)
                        .service(chats_controller::get_by_pair)
                        .service(chats_controller::get_all_chats)
                        .service(chats_controller::create_chat_pair)
                        .service(chats_controller::get_my_chat_pairs)
                    )
                    .service(
                        web::scope("circle")
                        .service(chats_controller::create_group_chat)
                        .service(chats_controller::get_circle)
                        .service(chats_controller::get_group_chats)
                    )
            )
            .service(user_controller::create_user)
            .service(user_controller::login_user)
            .service(user_controller::kura_id_signup)
            .service(user_controller::kura_id_login)
            .service(user_controller::get_code)
            .service(index)
            .service(system_controller::get_system_data)
            

            //


    })
        .bind(address)?
        .run()
        .await
}