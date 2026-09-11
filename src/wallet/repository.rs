use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use std::error::Error;
use mongodb::results::{InsertOneResult, UpdateResult};
use mongodb::Database;
use crate::wallet::models::Wallet;

const COLLECTION_NAME: &str = "Wallet";

pub struct WalletRepo;

impl WalletRepo {
    pub async fn create(db: &Database, data: &Wallet) -> Result<InsertOneResult, Box<dyn Error>> {
        let collection = db.collection::<Wallet>(COLLECTION_NAME);
        let res_diag = collection.insert_one(data).await;
        match res_diag {
            Ok(res_) => { return Ok(res_) }
            Err(err) => { return Err(err.into()) }
        }
    }

    pub async fn get_by_email(db: &Database, email: &String) -> Result<Option<Wallet>, Box<dyn Error>> {
        let filter = doc! {"user_email":email};
        let collection = db.collection::<Wallet>(COLLECTION_NAME);
        let mut wallet = collection.find_one(filter).await.ok().expect("Error getting test data");
        Ok(wallet)
    }

    pub async fn update(db: &Database, id: String, new_data: &Wallet) -> Result<UpdateResult, Box<dyn Error>> {
        let object_id = ObjectId::parse_str(id);
        let object_id = match object_id {
            Ok(object_id) => { object_id }
            Err(err) => { return Err(err.into()) }
        };
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<Wallet>(COLLECTION_NAME);
        let new_doc = doc! {
            "$set":{
                "amount":new_data.amount.to_owned(),
            }
        };
        let updated_doc = collection.update_one(filter, new_doc).await;

        match updated_doc {
            Ok(updated_doc) => { return Ok(updated_doc) }
            Err(err) => {
                return Err(err.into())
            }
        }
    }
}
