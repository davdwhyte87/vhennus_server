use std::env;

use dotenv::dotenv;
use mongodb::{Client, Database};
use mongodb::options::ClientOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::shared::config::CONFIG;

pub async fn init_db_pool_x() -> PgPool{
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

pub mod db{
    use mongodb::{options::ClientOptions, Client, Database};

    pub struct DB;

    impl DB { 

        pub fn say_hello(){}
        pub async fn initialize_db()->Result<Database, mongodb::error::Error>{
            // Parse a connection string into an options struct.
            let mut client_options = ClientOptions::parse("mongodb://localhost/hdos").await?;

            // Manually set an option.
            client_options.app_name = Some("hdos".to_string());

            // Get a handle to the deployment.
            let client = Client::with_options(client_options)?;
            for db_name in client.list_database_names().await? {
                println!("{}", db_name);
            }

            let db = client.database("hdos");
            return Result::Ok(db);
        }
        pub async fn xinitialize_db()->Result<(),mongodb::error::Error>{
            // Parse a connection string into an options struct.
            let mut client_options = ClientOptions::parse("mongodb://localhost/hdos").await?;
    
            // Manually set an option.
            client_options.app_name = Some("hdos".to_string());
    
            // Get a handle to the deployment.
            let client = Client::with_options(client_options)?;
            for db_name in client.list_database_names().await? {
                println!("{}", db_name);
            }

            let db = client.database("hdos");

            // List the names of the collections in that database.
            for collection_name in db.list_collection_names().await? {
                println!("{}", collection_name);
            }

            use mongodb::bson::{doc, Document};
            // Get a handle to a collection in the database.
            let collection = db.collection::<Document>("books");

            let docs = vec![
            doc! { "title": "1984", "author": "George Orwell" },
            doc! { "title": "Animal Farm", "author": "George Orwell" },
            doc! { "title": "The Great Gatsby", "author": "F. Scott Fitzgerald" },
            ];

            // Insert some documents into the "mydb.books" collection.
            collection.insert_many(docs).await?;

            Ok(())
           
        }  
    }
  
}

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