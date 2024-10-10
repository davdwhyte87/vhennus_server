

use std::{error::Error, os::windows::raw::SOCKET, vec};

use chrono::Datelike;
use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::{doc, from_document, Document}, results::InsertOneResult, Database};

use crate::{main, models::{buy_order::BuyOrder, message::OrderMessage, sell_order::SellOrder, trivia_game::TriviaGame, trivia_question::TriviaQuestion}};



pub const TRIVIA_QUESTION_COLLECTION:&str = "TriviaQuestion";
pub const TRIVIA_GAME_COLLECTION:&str = "TriviaGame";

pub struct  TriviaGameService{

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

        let todays_date = format!("{}/{}/{}", dateb.month(), dateb.month(), dateb.year());
        
        let filter = doc! {"date":todays_date.to_owned()};
        let pipeline = vec![
            doc! {
                "$lookup":{
                    "from": "TriviaQuestion",
                    "localField":"trivia_question_id",
                    "foreignField":"id",
                    "as":"trivia_question"
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
                    match from_document::<TriviaGame>(data){
                        Ok(data)=>{
                            games_.push(data);
                        },
                        Err(err)=>{
                            log::error!(" error converting document to struct  {}", err.to_string());
                        }
                    }
                },
                Err(err)=>{
                    log::error!(" error getting document from cursor {}", err.to_string());   
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
                let question = match Self::get_todays_question(db).await{
                    Ok(data)=>{data},
                    Err(err)=>{
                        log::error!(" error getting todays question  {}", err.to_string());
                        return Err(err.into());   
                    }
                };

                let ngame = TriviaGame{
                    id: uuid::Uuid::new_v4().to_string(),
                    trivia_question_id: question.id.to_owned(),
                    winner_user_name: None,
                    date: todays_date.to_owned(),
                    is_ended: false,
                    trivia_question: Some(question.to_owned())
                };
    
                //save this new game
                match Self::create_game(db, &ngame).await{
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

