

use std::{error::Error, vec};

use bigdecimal::BigDecimal;
use chrono::Datelike;
use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document, oid::ObjectId, Document}, options::UpdateOptions, results::{InsertOneResult, UpdateResult}, ClientSession, Database};

use crate::{controllers::trivia_game_controller::payout_winner, main, models::{buy_order::BuyOrder, message::OrderMessage, sell_order::SellOrder, trivia_game::TriviaGame, trivia_question::TriviaQuestion}};

use super::mongo_service::MongoService;



pub const TRIVIA_QUESTION_COLLECTION:&str = "TriviaQuestion";
pub const TRIVIA_GAME_COLLECTION:&str = "TriviaGame";

pub struct  TriviaGameService{

}


#[derive(strum_macros::Display, Debug)]
pub enum PlayTriviaError{
    WrongAnswer,
    CorrectButLate,
    OperationFailed,
    PayoutFailed
}
impl Error for PlayTriviaError {
    
}

impl TriviaGameService {
    pub async fn create_question(db:&Database, question:&TriviaQuestion)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<TriviaQuestion>(TRIVIA_QUESTION_COLLECTION);
  
        let result =collection.insert_one(question).await;

        let data = match result {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting into db  {}", err.to_string());
                return Err(err.into())
            }
        };
        Ok(data)
    }

    pub async fn create_game(db:&Database, game:&TriviaGame)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<TriviaGame>(TRIVIA_GAME_COLLECTION);
  
        let result =collection.insert_one(game).await;

        let data = match result {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error inserting into db  {}", err.to_string());
                return Err(err.into())
            }
        };
        Ok(data)
    }

 

    pub async fn update_game(db:&Database,id:String, game:&TriviaGame)->Result<UpdateResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.

        let collection = db.collection::<TriviaGame>(TRIVIA_GAME_COLLECTION);

        let query = doc! {"id":id };
        let update_data = doc! {"$set": doc! {
            "winner_user_name":game.winner_user_name.to_owned(),
            "is_ended":game.is_ended.to_owned()
        }};
        

       
        let result =collection.update_one(query, update_data).await;

        let data = match result {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error updating db  {}", err.to_string());
                return Err(err.into())
            }
        };
        Ok(data)
    }


    pub async fn update_question(db:&Database,id:String, question:&TriviaQuestion)->Result<UpdateResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.

        let collection = db.collection::<TriviaQuestion>(TRIVIA_QUESTION_COLLECTION);

        let query = doc! {"id":id };
        let update_data = doc! {"$set": doc! {
            "is_used":question.is_used.to_owned()
        }};
        let result =collection.update_one(query, update_data).await;

        let data = match result {
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error updating questions db  {}", err.to_string());
                return Err(err.into())
            }
        };
      
        Ok(data)
    }


     

    pub async fn answer_question(db:&Database, ans:String,
        wallet_address:String, user_name:String,
        amount:BigDecimal
       )->Result<(), Box<dyn Error>>{
       let mut  session = match db.client().start_session().await{
           Ok(data)=>{data},
           Err(err)=>{
               log::error!("error creating database session {}", err);
               return Err(err.into());
           }
       };

       match session.start_transaction().await{
           Ok(data)=>{},
           Err(err)=>{
               log::error!("error creating session transaction  {}", err);
               return Err(err.into());   
           }
       };

       let game_collection = session.client().database(&MongoService::get_db_name()).collection::<TriviaGame>(TRIVIA_GAME_COLLECTION);
       let question_collection = session.client().database(&MongoService::get_db_name()).collection::<TriviaQuestion>(TRIVIA_QUESTION_COLLECTION);

       // logic
       // 

       let game = match Self::get_todays_game(db).await{
           Ok(data)=>{data},
           Err(err)=>{
               log::error!("error getting todays game {}", err);
               return Err(Box::from(PlayTriviaError::OperationFailed))
           }
       }; 
       let mut new_game = game.clone();

       let question =match  game.trivia_question{
           Some(data)=>{data}, 
           None=>{
               log::error!(" error getting question");
               return Err(Box::from(PlayTriviaError::OperationFailed))
           }
       };
       

       if ans == question.answer {
           // check if the game is ended
           if game.is_ended {
               return Err(Box::from(PlayTriviaError::CorrectButLate))
           }else{
               // end the game
               //let mut new_game = game.clone();
               new_game.is_ended = true;
               new_game.winner_user_name = Some(user_name);
               let query = doc! {"id":new_game.id};
               let update_data = doc! {"$set": doc! {
                   "winner_user_name":new_game.winner_user_name.to_owned(),
                   "is_ended":new_game.is_ended.to_owned()
               }};
               match game_collection.update_one(query, update_data).session(&mut session).await {
                   Ok(data)=>{
                       // payout the user from trivia wallet
                       match payout_winner(wallet_address, amount ).await{
                           Ok(data)=>{},
                           Err(err)=>{
                               log::error!(" error with blockchain transaction {}", err);
                               match session.abort_transaction().await{
                                   Ok(x)=>{},
                                   Err(err)=>{log::error!("abort error {}", err)}
                               };
                               return Err(Box::from(PlayTriviaError::PayoutFailed))     
                           }
                       };    
                   },
                   Err(err)=>{
                       log::error!(" error updating game {}", err);
                       session.abort_transaction().await;
                       return Err(Box::from(PlayTriviaError::OperationFailed))   
                   }
               }

               // update the question to be used
               let mut new_question = question.clone();
               new_question.is_used = true;
               let query = doc! {"id":new_question.id.to_owned() };
               let update_data = doc! {"$set": doc! {
                   "is_used":new_question.is_used.to_owned()
               }};
               match question_collection.update_one(query, update_data).session(&mut session).await{
                    Ok(data)=>{},
                    Err(err)=>{
                        log::error!(" error updating question data {}", err);
                        session.abort_transaction().await;
                        return Err(Box::from(PlayTriviaError::OperationFailed))
                    }
                };
            }
       }else{
           // wrong answer 
           
           return Err(Box::from(PlayTriviaError::WrongAnswer))                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               
       }

       session.commit_transaction().await;
       return Ok(());
   }



    

    pub async fn get_all(db:&Database)->Result<Vec<TriviaQuestion>, Box<dyn Error> >{
        let collection = db.collection::<TriviaQuestion>(TRIVIA_QUESTION_COLLECTION);

        let mut result =match  collection.find(doc!{}).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error getting data  {}", err.to_string());
                return Err(err.into());
            }
        };

        let mut questions:Vec<TriviaQuestion> = vec![];
        while let Some(x) = result.next().await{
            let data = match x{
                Ok(data)=>{data},
                Err(err)=>{
                    log::error!(" error getting data itr  {}", err.to_string());
                    return Err(err.into());
                }
            };
            questions.push(data);

        };
        
         return Ok(questions)
    }


    pub async fn get_todays_question(db:&Database)->Result<TriviaQuestion, Box<dyn Error> >{
        let collection = db.collection::<TriviaQuestion>(TRIVIA_QUESTION_COLLECTION);

        let mut result =match  collection.find(doc!{}).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error getting data  {}", err.to_string());
                return Err(err.into());
            }
        };

        let mut questions:Vec<TriviaQuestion> = vec![];
        while let Some(x) = result.next().await{
            let data = match x{
                Ok(data)=>{data},
                Err(err)=>{
                    log::error!(" error getting data itr  {}", err.to_string());
                    return Err(err.into());
                }
            };
            questions.push(data);

        };

        // loop throught to get todays question
        for question in questions{
            if question.is_used == false{
                return Ok(question)
            }
        }
        return Err(Box::from("No question available"))
    }


    pub async fn get_todays_game(db:&Database)->Result<TriviaGame, Box<dyn Error>>{
       
        // get trivia game with todays date 
        // get toddays date 
        let dateb = chrono::Utc::now();

        let todays_date = format!("{}/{}/{}", dateb.month(), dateb.day(), dateb.year());
        
        let filter = doc! {"date":todays_date.to_owned()};
        let pipeline = vec![
            doc! {
                "$match":filter
            },
            doc! {
                "$lookup":{
                    "from": "TriviaQuestion",
                    "localField":"trivia_question_id",
                    "foreignField":"id",
                    "as":"trivia_question"
                }
            },
            doc! {
                "$unwind": {
                    "path": "$trivia_question",
                    "preserveNullAndEmptyArrays": true
                }
            }
        ];

        let collection = db.collection::<TriviaGame>(TRIVIA_GAME_COLLECTION);
        let mut result =match  collection.aggregate(pipeline).await{
            Ok(data)=>{data},
            Err(err)=>{
                log::error!(" error getting data  {}", err.to_string());
                return Err(err.into());
            }
        };

        let mut games_:Vec<TriviaGame> = vec![];

        while let Some(games_data)= result.next().await{
            match games_data {
                Ok(data)=>{
                    log::debug!("data {}", data);
                    match from_document::<TriviaGame>(data){

                        Ok(data)=>{

                            games_.push(data);
                        },
                        Err(err)=>{
                            log::error!(" error converting document to struct  {}", err.to_string());
                            return Err(err.into());
                        }
                    }
                },
                Err(err)=>{
                    log::error!(" error getting document from cursor {}", err.to_string());
                    return Err(err.into());   
                }
            }
        }

        // get the first data because we are expecting one due to the filter

        let game = games_.get(0);
        // todays game does not exist and this is probably the first player
        // create a game for today 
        match game {
            Some(data)=>{
                return Ok(data.to_owned())
            },
            None=>{
               // if there is no game, create new one 
                let question = match Self::get_todays_question(&db).await{
                    Ok(data)=>{data},
                    Err(err)=>{
                        log::error!(" error getting todays question  {}", err.to_string());
                        return Err(err.into());   
                    }
                };

                let ngame = TriviaGame{
                    mongo_id:ObjectId::new(),
                    id: uuid::Uuid::new_v4().to_string(),
                    trivia_question_id: question.id.to_owned(),
                    winner_user_name: None,
                    date: todays_date.to_owned(),
                    is_ended: false,
                    trivia_question: Some(question.to_owned())
                };
    
                //save this new game
                match Self::create_game(&db, &ngame).await{
                    Ok(data)=>{data},
                    Err(err)=>{
                        log::error!(" error creating game  {}", err.to_string());
                        return Err(err.into()); 
                    }
                };

                return Ok(ngame)
            }
        }
    
  
         
        
        // let game = match result {
        //     Some(data)=>{data},
        //     None=>{

       
        //     }
        // };


        
    }

}

