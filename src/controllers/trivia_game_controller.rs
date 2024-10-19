
use std::env;

use actix_web::{get, post, web::{self, post, Data, ReqData}, HttpResponse};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::{models::{payment_method::{PaymentMethod, PaymentMethodData}, post, request_models::TransferReq, response::{ GenericResp, Response}, sell_order::{self, Currency, SellOrder}, system::System, trivia_game::TriviaGame}, req_models::{create_payment_method_req::CreatePaymentMethodReq, create_sell_order_req::{AnswerGame, CreatePostReq, CreateSellOrderReq, UpdateSellOrderReq}}, services::{mongo_service::MongoService, payment_method_service::PaymentMethodService, sell_order_service::SellOrderService, system_service::SystemService, tcp::send_to_tcp_server, trivia_game_service::{PlayTriviaError, TriviaGameService}}, utils::{auth::Claims, formatter}};



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


#[post("/play")]
pub async fn play_game(
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>,
    req: Result<web::Json<AnswerGame>, actix_web::Error>,
)->HttpResponse{
    // validate input 
    let mut respData = GenericResp::<String>{
        message:"".to_string(),
        server_message: None,
        data: None
    };

    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData); 
        }
    };

    // user claim
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // get system configs
    let system_data = match SystemService::get_system_data(&database.db).await{
        Ok(data)=>{
            match data {
                Some(data)=>{data},
                None=>{
                    respData.message = "System data not found".to_string();
                    respData.server_message = None;
                    respData.data = None;
                    return HttpResponse::InternalServerError().json(respData)   
                }
            }
        },
        Err(err)=>{
            log::error!("Error getting system data {}", err);
            respData.message = "error getting system data".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };


    match TriviaGameService::answer_question(&database.db, req.answer.clone(), req.wallet_address.clone(),
         claim.user_name.to_owned(), system_data.trivia_win_amount).await{
            Ok(data)=>{},
            Err(err)=>{
               if let Some(play_error) = err.downcast_ref::<PlayTriviaError>(){
                match play_error{
                    PlayTriviaError::CorrectButLate=>{
                        respData.message = "Correct, but somebody beat you to it. Try again next time".to_string();
                        respData.server_message = Some(err.to_string());
                        respData.data = Some("CSW".to_string());
                        return HttpResponse::Ok().json(respData)
                    }
                    PlayTriviaError::OperationFailed=>{
                        respData.message = "Operation failed".to_string();
                        respData.server_message = Some(err.to_string());
                        respData.data = None;
                        return HttpResponse::InternalServerError().json(respData)
                    }
                    PlayTriviaError::PayoutFailed=>{
                        respData.message = "Payout failed".to_string();
                        respData.server_message = Some(err.to_string());
                        respData.data = None;
                        return HttpResponse::InternalServerError().json(respData)
                    }
                    PlayTriviaError::WrongAnswer=>{
                        respData.message = "Wrong answer".to_string();
                        respData.server_message = Some(err.to_string());
                        respData.data = Some("W".to_string());
                        return HttpResponse::Ok().json(respData)
                    }
                }
               }
            }
         };

    respData.message = "Congratulations you win".to_string();
    respData.server_message = None;
    respData.data = Some("C".to_string());
    return HttpResponse::Ok().json(respData)

}
// pub async fn play_game2(
//     database:Data<MongoService>,
//     claim:Option<ReqData<Claims>>,
//     req: Result<web::Json<AnswerGame>, actix_web::Error>,
// )->HttpResponse{
//     // validate input 
//     let mut respData = GenericResp::<String>{
//         message:"".to_string(),
//         server_message: None,
//         data: None
//     };

//     let req = match req {
//         Ok(data)=>{data},
//         Err(err)=>{
//             log::error!("validation  error  {}", err.to_string());
//             respData.message = "Validation error".to_string();
//             respData.server_message = Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::InternalServerError().json( respData); 
//         }
//     };

//     // user claim
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             respData.message = "Unauthorized".to_string();
//             respData.server_message = None;
//             respData.data = None;
//             return HttpResponse::Unauthorized()
//                 .json(
//                     respData
//                 )
//         }
//     };

//     // get system configs
//     let system_data = match SystemService::get_system_data(&database.db).await{
//         Ok(data)=>{
//             match data {
//                 Some(data)=>{data},
//                 None=>{
//                     respData.message = "System data not found".to_string();
//                     respData.server_message = None;
//                     respData.data = None;
//                     return HttpResponse::InternalServerError().json(respData)   
//                 }
//             }
//         },
//         Err(err)=>{
//             log::error!("Error getting system data {}", err);
//             respData.message = "error getting system data".to_string();
//             respData.server_message = Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::InternalServerError().json(respData)
//         }
//     };

//      // fetch todays game from the database 
//     let game = match TriviaGameService::get_todays_game(&database.db).await{
//         Ok(data)=>{data},
//         Err(err)=>{
//             log::error!(" error getting trivia game  {}", err.to_string());
//             respData.message = "error getting data".to_string();
//             respData.server_message = Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::InternalServerError().json(respData)
//         }
//     }; 
//     let mut new_game = game.clone();

//     // question
//     let question =match  game.trivia_question{
//         Some(data)=>{data}, 
//         None=>{
//             log::error!(" error getting question");
//             respData.message = "error getting question from db".to_string();
//             respData.server_message = None;
//             respData.data = None;
//             return HttpResponse::NotFound().json(respData)    
//         }
//     };

//     // check if the answer is correct
//     if req.answer == question.answer {
//         // check if the game is ended
//         if game.is_ended {
//             respData.message = "Correct, but someone beat you to it".to_string();
//             respData.server_message = None;
//             respData.data = Some("0".to_string());
//             return HttpResponse::Ok().json(respData)  
//         }else{
//             // end the game
//             //let mut new_game = game.clone();
//             new_game.is_ended = true;
//             new_game.winner_user_name = Some(claim.user_name.to_owned());
//             match TriviaGameService::update_game(&database.db, game.id, &new_game).await {
//                 Ok(data)=>{
//                     // payout the user from trivia wallet
//                     match payout_winner(req.wallet_address.to_owned(), system_data.trivia_win_amount ).await{
//                         Ok(data)=>{},
//                         Err(err)=>{
//                             log::error!("Error paying user {}", err.to_string());
//                             respData.message = "Error paying out user".to_string();
//                             respData.server_message = Some(err.to_string());
//                             respData.data = None;
//                             return HttpResponse::InternalServerError().json(respData)    
//                         }
//                     };

//                     respData.message = "Congratulations you have won todays game".to_string();
//                     respData.server_message = None;
//                     respData.data = Some("1".to_string());
//                     return HttpResponse::Ok().json(respData)     
//                 },
//                 Err(err)=>{
//                     log::error!(" error updating game  {}", err.to_string());
//                     respData.message = "error saving play".to_string();
//                     respData.server_message = Some(err.to_string());
//                     respData.data = None;
//                     return HttpResponse::InternalServerError().json(respData)  
//                 }
//             }
//         }
//     }else{
//         // wrong answer 
//         respData.message = "Sorry, wrong answer".to_string();
//         respData.server_message = None;
//         respData.data = Some("0".to_string());
//         return HttpResponse::Ok().json(respData)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              
//     }
// }

pub fn payout_player(){

}
pub async fn payout_winner(address:String, amount:BigDecimal)->Result<(),Box<dyn std::error::Error>>{
     

    // send message to the kuracoin blockchain to create new user
    let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            println!("{}", err.to_string());
            return  Err(Box::from("Error connecting to blockchain"));
        }
    };

    let wallet = match  env::var("TRIVIA_WALLET"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            println!("{}", err.to_string());
            return  Err(Box::from("Error connecting to blockchain"));
        }
    };

    let wallet_password = match  env::var("TRIVIA_WALLET_PASSWORD"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            println!("{}", err.to_string());
            return  Err(Box::from("Error connecting to blockchain"));
        }
    };

    let message_data = match serde_json::to_string(&TransferReq{
        sender: wallet,
        receiver: address,
        amount: amount,
        transaction_id: Uuid::new_v4().to_string(),
        sender_password: wallet_password
    }){
        Ok(data)=>{data},
        Err(err)=>{
            println!("{}", err.to_string());
            return  Err(Box::from("Error parsing data"));
        }
    };
    let message = formatter::Formatter::request_formatter(
        "Transfer".to_string(), 
        message_data,
        "".to_string(), 
        "".to_string(),
        "0".to_string());

    let m = message.clone();
    let ip = kura_coin_server_ip.clone();
    let result = web::block(move || send_to_tcp_server(m,ip  )).await;
    let response_string =match result {
        Ok(data)=>{
            match data {
                Ok(data)=>{data},
                Err(err)=>{
                    println!("{}", err.to_string());
                    return  Err(Box::from("Error parsing data"));    
                }
            }
        },
        Err(err)=>{ 
            println!("{}", err.to_string());
            return  Err(Box::from("Error parsing data")); 
        }
    };

    let resp_data: Vec<&str>= response_string.split('\n').collect();
    let code = match resp_data.get(0){
        Some(data)=>{data},
        None=>{
            return  Err(Box::from("Error decoding data from blockchain server"));    
        }
    };

    if *code != "1"{
        // blockchain request failed
        return  Err(Box::from("Failed trnasfer on blockchain"));   
    }

    return Ok(())

}
