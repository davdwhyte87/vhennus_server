use mongodb::{Client, options::ClientOptions};

pub mod db{
    use mongodb::{options::ClientOptions, Client, Database};
    // use r2d2_mongodb::mongodb::Client;
    // use rocket::http::hyper::Error;


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