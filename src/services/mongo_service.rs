use std::env;

use mongodb::{Client, Database};
use mongodb::options::ClientOptions;
use r2d2_mongodb::mongodb::db;

pub struct MongoService{
    pub db:Database,
    pub client:Client
}


impl MongoService{
   // pub async fn  init()->MongoService{
   //      let mongo_url = match env::var("MONGO_URL"){
   //          Ok(data)=>{data},
   //          Err(err)=>{
   //              log::error!("error getting mongo url var {}", err.to_string());
   //              panic!();
   //          }
   //      };
   // 
   //      let app_env = match env::var("APP_ENV"){
   //          Ok(data)=>{data},
   //          Err(err)=>{
   //              log::error!("error getting mongo url var {}", err.to_string());
   //              panic!();
   //          }
   //      };
   //     // Parse a connection string into an options struct.
   //     //let mut client_options = ClientOptions::parse(mongo_url).await.unwrap();
   // 
   //     // Manually set an option.
   //     let mut db_name = self::MongoService::get_db_name();
   //    
   //     //client_options.app_name = Some(db_name.to_string());
   // 
   //     // Get a handle to the deployment.
   //     //let client = Client::with_options(client_options).unwrap();
   //     //let db = &client.database(&db_name);
   // 
   //     return MongoService{db: db.clone(), client: client}
   // }
   pub fn get_db_name()->String{
    let app_env = match env::var("APP_ENV"){
        Ok(data)=>{data},
        Err(err)=>{
           "local".to_owned()
        }
    };
    let mut db_name = "vhennus_local".to_string();
    if app_env =="test"{
        db_name = "vhennus_test".to_owned()
    }
    if app_env == "local"{
        db_name = "vhennus_test".to_owned()
    }
    if app_env == "prod" {
        db_name = "vhennus".to_owned()
    }

       return db_name;
    }

}