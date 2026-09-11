use mongodb::results::InsertOneResult;
use serde::Serialize;
use serde_derive::Deserialize;


#[derive(Serialize)]
pub struct Response {
    pub message: String,

}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BResponse<T> {
    pub status: i32,
    pub message: String,
    pub data: Option<T>,
}



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GenericResp<T> {
    pub message: String,
    pub server_message: Option<String>,
    pub data:Option<T>
}






#[derive(Serialize)]
pub struct ResponseInsert{
    pub message: String,
    pub data: InsertOneResult
}

#[derive(Serialize)]
pub struct LoginResp{
    pub message: String,
    pub token: String
}

#[derive(Serialize)]
pub struct CodeResp{
    pub code: i32
}