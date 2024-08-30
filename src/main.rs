use std::{env, fmt};
use actix_web::error::JsonPayloadError;
use actix_web::{error, get, web, App, Error, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web::web::{resource, route, service, Data, JsonConfig};


mod controllers;
use controllers::buy_order_controller::seller_confirmed;
use controllers::{
    buy_order_controller, order_message_controller, payment_method_controller, player_controller, post_controller, power_ups_controller, sell_order_controller, user_controller, wallet_controller

};
mod models;
use dotenv::dotenv;
use models::{response};
mod database;
use database::db::db;
mod services;
use serde_json::json;
use services::{user_service, pet_service, diagnosis_service};
use thiserror::Error;
use crate::services::mongo_service::MongoService;
mod utils;
mod req_models;
mod middlewares;





#[get("/")]
async fn index() -> impl Responder {
    "Hello, Bread!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {


    dotenv().ok();

    env::set_var("RUST_BACKTRACE", "full");
    let db = MongoService::init().await;
    let db_data = Data::new(db);

    struct ApiError{
       

    }
    impl ApiError {

        pub fn json_error(cfg: JsonConfig) -> JsonConfig {
            cfg.limit(4096).error_handler(|err: JsonPayloadError, _req| {
                // create custom error response
                error::InternalError::from_response(
                    format!("JSON error: {:?}", err),
                    HttpResponse::from_error(err),
                ).into()
            })
        }
    }

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
                    .service(player_controller::update_player_stats)
                    .service(player_controller::get_player_stats)
                    .service(player_controller::add_account_details)
                    .service(wallet_controller::buy_coin)
                    .service(wallet_controller::get_wallet)
                    .service(power_ups_controller::buy_power_up)
                    .service(power_ups_controller::use_power_up)
                    .service(power_ups_controller::get_player_powerups)

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
                    )
            )
            .service(user_controller::create_user)
            .service(user_controller::login_user)
            .service(power_ups_controller::use_power_up)
            .service(user_controller::kura_id_signup)
            .service(user_controller::kura_id_login)
            .service(user_controller::get_code)
            .service(hello)
            

            //


    })
        .bind(("127.0.0.1", 80))?
        .run()
        .await
}