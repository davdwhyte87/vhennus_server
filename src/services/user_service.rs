use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::string::ToString;
use handlebars::Handlebars;
use mongodb::{Client, Database, options::ClientOptions};
use mongodb::bson::{doc, Document};
// use mongodb::bson::extjson::de::Error;
use std::error::Error;
use mongodb::bson::oid::ObjectId;
use mongodb::results::{InsertOneResult, UpdateResult};
use r2d2_mongodb::mongodb::ErrorCode::OK;
use serde_json::{json, Value};


use crate::database::db::db::DB;
use crate::models::helper::EmailData;
use crate::models::request_models::LoginReq;
use crate::models::user::User;
use crate::utils::send_email::{ACTIVATE_EMAIL, get_body, send_email};

pub const USER_COLLECTION:&str = "User";

pub struct UserService{
    client: Client

}

impl UserService{
    pub async fn create_user(db:&Database, user:&User)->Result<InsertOneResult, Box<dyn Error>>{
        // Get a handle to a collection in the database.
        let collection = db.collection::<User>(USER_COLLECTION);

        // let new_user = User{
        //     id:None,
        //     name:user.name
        // };

        //send email
        // let mut file = File::open("html/activate.html").expect("File not found");
        // let mut html_data = String::new();
        // file.read_to_string(&mut html_data);
        let code:u32= 9384;


        // let name = user.name.as_str().to_string();
        // 
        // let mut reg = Handlebars::new();
        // let order_email_content = reg.render_template (
        //     include_str!("../utils/html/activate_new_account.hbs"),
        //     &serde_json::json!({"name" :name, "code":code})).unwrap();
        // 
        // let email_data = EmailData{
        //     subject:"Confirmation code".to_string(),
        //     to: (*user.email).parse().unwrap(),
        //     body: order_email_content
        // };
        // send_email(email_data);
        // Insert data into db.
        let res_user =collection.insert_one(user).await;

        let res_user = match res_user {
            Ok(res_user)=>{res_user},
            Err(err)=>{return Err(err.into())}
        };
        Ok(res_user)
    }



    pub async fn get_by_id(db:&Database, id:String)->Result<Option<User>, Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<User>(USER_COLLECTION);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    pub async fn get_by_email(db:&Database, email:String)->Result<Option<User>, Box<dyn Error>>{

        let filter = doc! {"email":email};
        let collection = db.collection::<User>(USER_COLLECTION);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    
    pub async fn get_by_(db:&Database, filter:Document)->Result<Option<User>, Box<dyn Error>>{

        let collection = db.collection::<User>(USER_COLLECTION);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_detail)=>{return Ok(user_detail)},
            Err(err)=>{return Err(err.into())}
        };
    }

    pub async fn update(
        db:&Database,
        email:&String,
        mut new_data:&User
    )
        ->Result<UpdateResult, Box<dyn Error>>
    {
        let filter = doc! {"email":email};
        let collection = db.collection::<User>(USER_COLLECTION);
        let new_doc = doc! {
            "$set":{
                "code":new_data.code.to_owned(),
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

