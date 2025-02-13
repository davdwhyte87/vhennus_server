

use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::string::ToString;
use handlebars::Handlebars;
use mongodb::{Client, Database, options::ClientOptions};
use mongodb::bson::doc;
// use mongodb::bson::extjson::de::Error;
use std::error::Error;
use mongodb::bson::oid::ObjectId;
use mongodb::results::{InsertOneResult, UpdateResult};
use r2d2_mongodb::mongodb::ErrorCode::OK;
use serde_json::{json, Value};


use crate::database::db::db::DB;
use crate::models::account_details::AccountDetails;
use crate::models::helper::EmailData;
use crate::models::request_models::LoginReq;
use crate::models::user::User;
// use crate::utils::send_email::{ACTIVATE_EMAIL, get_body, send_email};

const COLLECTION_NAME:&str = "Account Detail";

pub struct AccountDetailsService{
    client: Client

}

impl AccountDetailsService{
    pub async fn create(db:&Database, data:&AccountDetails)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<AccountDetails>(COLLECTION_NAME);

        let res_user =collection.insert_one(data).await;

        let res_user = match res_user {
            Ok(res_user)=>{res_user},
            Err(err)=>{return Err(err.into())}
        };
        Ok(res_user)
    }



    pub async fn get_by_id(db:&Database, id:String)->Result<Option<AccountDetails>, Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<AccountDetails>(COLLECTION_NAME);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    pub async fn get_by_email(db:&Database, email:String)->Result<Option<AccountDetails>, Box<dyn Error>>{

        let filter = doc! {"user_email":email};
        let collection = db.collection::<AccountDetails>(COLLECTION_NAME);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    pub async fn update(
        db:&Database,
        email:&String,
        mut new_data:&AccountDetails
    )
        ->Result<UpdateResult, Box<dyn Error>>
    {
        let filter = doc! {"user_email":email};
        let collection = db.collection::<AccountDetails>(COLLECTION_NAME);
        let new_doc = doc! {
            "$set":{
                "account_name":new_data.account_name.to_owned(),
                "account_number": new_data.account_number.to_owned(),
                "bank_name":new_data.bank_name.to_owned(),
                "updated_at":new_data.updated_at.to_owned()

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
}

