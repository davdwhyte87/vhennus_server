
use actix_web::{get, web::Data, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{payment_method::{PaymentMethod, PaymentMethodData}, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}, system::System, trivia_game::TriviaGame}, req_models::{create_payment_method_req::CreatePaymentMethodReq, create_sell_order_req::{CreateSellOrderReq, UpdateSellOrderReq}}, services::{mongo_service::MongoService, payment_method_service::PaymentMethodService, sell_order_service::SellOrderService, system_service::SystemService, trivia_game_service::{TriviaGameService}}, utils::auth::Claims};



#[get("/todays_game")]
pub async fn get_todays_game(

    database:Data<MongoService>
)->HttpResponse{
    let mut respData = GenericResp::<TriviaGame>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };

    // fetch todays game from the database 
    let game = match TriviaGameService::get_todays_game(&database.db).await{
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error getting trivia game  {}", err.to_string());
            respData.message = "error getting data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };


    // return game
    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(game);
    return HttpResponse::Ok().json(respData)
}