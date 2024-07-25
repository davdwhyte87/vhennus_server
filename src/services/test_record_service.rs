use std::env::current_exe;
use mongodb::bson::{bson, doc, from_bson, from_document};
// use mongodb::bson::extjson::de::Error;
use std::error::Error;
use mongodb::bson::oid::ObjectId;
use mongodb::Database;
use std::error::Error as XError ;
use mongodb::results::{InsertOneResult, UpdateResult};
use crate::models::test_record::TestRecord;
use mongodb::bson::extjson::de::Error::DeserializationError;
use futures::stream::TryStreamExt;
use futures::StreamExt;

const COLLECTION_NAME:&str = "Test Record";

pub struct TestRecordService{

}

impl TestRecordService {
    pub async fn create(db: &Database, test_record: TestRecord) -> Result<InsertOneResult,  Box<dyn Error>> {
        // Get a handle to a collection in the database.
        let collection = db.collection::<TestRecord>(COLLECTION_NAME);
        let res_diag =collection.insert_one(test_record).await;
        match res_diag {
            Ok(res_)=>{return Ok(res_)},
            Err(err)=>{return Err(err.into())}
        }
    }

    pub async fn get_by_user_id(db:&Database, id:String)->Result<Vec<TestRecord>,  Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id);
        let object_id = match object_id {
            Ok(object_id)=>{object_id},
            Err(error)=>{
                return Err(error.into())
            }
        };
        let filter = doc! {"patient_id":object_id};
        let collection = db.collection::<TestRecord>(COLLECTION_NAME);
        let mut cursor = collection.find(filter).await;
        let mut cursor = match cursor {
            Ok(cursor)=>{cursor},
            Err(err)=>{return Err(err.into())}
        };
        let mut test_record:Vec<TestRecord> = Vec::new();

        while let Some(diag)=  match cursor.try_next().await {
            Ok(cursor) => {cursor}
            Err(err) => {return Err(err.into())}
        } {
            test_record.push(diag);
        }
        Ok(test_record)
    }

    pub async fn get_by_id(db:&Database, id:String)->Result<Option<TestRecord>,  Box<dyn Error>>{
        let object_id = ObjectId::parse_str(id);
        let object_id = match object_id {
            Ok(object_id)=>{object_id},
            Err(error)=>{ return Err(error.into())}
        };
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<TestRecord>(COLLECTION_NAME);
        let user_detail = collection.find_one(filter).await;
        match user_detail {
            Ok(user_details)=>{
              return Ok(user_details)
            },
            Err(err)=>{
                return Err(err.into())
            }
        }
    }


    pub async fn get_all_records(db:&Database)->Result<Vec<TestRecord>, Box<dyn XError>>{
        // let object_id = ObjectId::parse_str(id).unwrap();
        // let filter = doc! {"_id":object_id};
        let collection = db.collection::<TestRecord>(COLLECTION_NAME);
        let lookup = doc!{
            "$lookup": {
                "from": "Test Data",
                "let": { "test_record_id": "$_id" },
                "pipeline": [
                {
                    "$match": {
                    "$expr": {
                        "$and": [
                        { "$in": [ "$$test_record_id", "test_datas" ] }
                        ]
                    }
                }
                }
                ],
                "as": "test_data"
            }
        };

        let lookup_2 = doc! {
             "$lookup":
                {
                   "from": "Test Data",
                   "localField": "test_datas",
                   "foreignField": "_id",
                   "as": "test_data"
                }
        };
        // let projection_pipeline = doc!
        //     {
        //         "$project": {
        //         "_id: 1,
        //         name: 1,
        //         favorite: { $eq: [ { $size: "$users" }, 1 ] }
        //     }


        let mut results = collection.aggregate(vec![lookup_2]).await?;
        let mut test_records:Vec<TestRecord> = Vec::new();
        while let Some(result) = results.next().await{
            let data: TestRecord= from_document(result?)?;
            test_records.push(data);
        }
        return Ok(test_records);

    }
    

    pub async fn update(db:&Database, id:String, mut new_data:&TestRecord)->Result<UpdateResult, Box<dyn XError>>{
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id":object_id};
        let collection = db.collection::<TestRecord>(COLLECTION_NAME);
        let new_doc = doc! {
            "$set":{
                "test_datas":new_data.test_datas.to_owned()
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


    pub async fn get_by_patient_email(db:&Database, email:String)->Result<Vec<TestRecord>,  Box<dyn Error>>{
        // if email.is_empty(){
        //     return Err((_))
        // }
        let lookup_2 = doc! {
             "$lookup":
                {
                   "from": "Test Data",
                   "localField": "test_datas",
                   "foreignField": "_id",
                   "as": "test_data"
                }
        };

        let filter = doc! {"$match":{"patient_email":email}};
        let collection = db.collection::<TestRecord>(COLLECTION_NAME);
        // let mut cursor = collection.find(filter, None).await;
        let mut cursor = collection.aggregate(vec![filter,lookup_2 ]).await;
        let mut cursor = match cursor {
            Ok(cursor)=>{cursor},
            Err(err)=>{return Err(err.into())}
        };
        let mut test_record:Vec<TestRecord> = Vec::new();

        while let Some(diag)=  match cursor.try_next().await {
            Ok(cursor) => {cursor}
            Err(err) => {return Err(err.into())}
        } {
            let data: TestRecord= from_document(diag)?;
            test_record.push(data);
        }
        Ok(test_record)
    }
}

