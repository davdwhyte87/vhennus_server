use std::fmt::{Debug};
use actix_web::App;
use mongodb::bson::doc;
use std::error::Error;
// use mongodb::bson::extjson::de::Error;
use mongodb::bson::oid::ObjectId;
use mongodb::Database;
use mongodb::results::{InsertOneResult, UpdateResult};
use crate::models::test_record::TestRecord;
use mongodb::bson::extjson::de::Error::DeserializationError;
use futures::stream::TryStreamExt;
use mongodb::change_stream::event::ResumeToken;
use mongodb::error::{ErrorKind};
use r2d2_mongodb::mongodb::Error::OperationError;
use crate::models::test_data::TestData;
use crate::models::wallet::Wallet;

const COLLECTION_NAME:&str = "Wallet";


pub struct WalletService {

}



impl WalletService{
    pub async fn get_by_id(db:&Database, id:String)->Result<Option<TestData>, Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<TestData>(COLLECTION_NAME);
        let user_detail = collection.find_one(filter).await;
        let user_detail = match user_detail {
            Ok(user_detail)=>{
                user_detail
            },
            Err(err)=>{return Err(err.into())}
        };
        Ok(user_detail)
    }

    pub async fn create(db: &Database, data: &Wallet) -> Result<InsertOneResult, Box<dyn Error>> {
        // Get a handle to a collection in the database.
        let collection = db.collection::<Wallet>(COLLECTION_NAME);
        let res_diag =collection.insert_one(data).await;
        match res_diag {
            Ok(res_)=>{return Ok(res_)},
            Err(err)=>{return Err(err.into())}
        }
    }



    pub async fn get_by_user_id(db:&Database, id:String)->Result<Vec<TestData>, Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id);
        let object_id = match object_id {
            Ok(object_id)=>{object_id},
            Err(error)=>{ return Err(error.into())}
        };
        let filter = doc! {"patient_id":object_id};
        let collection = db.collection::<TestData>(COLLECTION_NAME);
        let mut cursor = collection.find(filter).await.ok().expect("Error getting test data");
        let mut diagnosis:Vec<TestData> = Vec::new();

        while let Some(diag)= match cursor.try_next().await {
            Ok(cursor) => {cursor}
            Err(err) => {return Err(err.into())}
        } {
            diagnosis.push(diag);
        }
        Ok(diagnosis)
    }



    pub async fn get_by_email(db:&Database, email:&String)->Result<Option<Wallet>, Box<dyn Error>>{
      
        let filter = doc! {"user_email":email};
        let collection = db.collection::<Wallet>(COLLECTION_NAME);
        let mut wallet = collection.find_one(filter).await.ok().expect("Error getting test data");
        // let wallet = match wallet {
        //     Some(wallet)=>{wallet},
        //     None=> {
        //         return Err(Box::from("No wallet found"))
        //     }
        //
        // };
        Ok(wallet)
    }
    pub async fn update(db:&Database, id:String, mut new_data:&Wallet)->Result<UpdateResult, Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id);
        let object_id = match object_id {
            Ok(object_id)=>{object_id},
            Err(err)=>{return Err(err.into())}
        };
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<TestData>(COLLECTION_NAME);
        let new_doc = doc! {
            "$set":{
                "amount":new_data.amount.to_owned(),
            }
        };
        let updated_doc = collection.update_one(filter,new_doc )
            .await;

        match updated_doc {
            Ok(updated_doc)=>{return Ok(updated_doc)},
            Err(err)=>{
                return Err(err.into())
            }
        }
    }

    // each test record has many related testdata, this function gets all test data for a given 
    //test record 
    pub async fn get_all_test_data_for_test_record(db:&Database, id:String){

    }
}