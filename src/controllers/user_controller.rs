use std::borrow::{Borrow, BorrowMut};
use std::env;
use std::future::IntoFuture;
use actix_web::{Responder, get, HttpResponse, web::Json, post};

use actix_web::web::{self, Data, ReqData};
use handlebars::Handlebars;
use mongodb::bson::doc;
use rand::Rng;
use regex::Replacer;
use serde::Deserialize;
use validator::Validate;
use crate::database::db::db::DB;
use crate::models::fried_request::{FriendRequest, FriendRequestStatus};
use crate::models::helper::EmailData;
use crate::models::power_up::{PlayerPowerUp, PowerUpType};
use crate::models::request_models::{CreateKuracoinID, GetCodeReq, LoginReq, SendFriendReq};
use crate::models::response::{CodeResp, GenericResp, LoginResp, PlayerRunInfoRes, Response};
use crate::models::run_info::RunInfo;
use crate::models::user::{User, UserType};
use crate::models::wallet::Wallet;
use crate::req_models::create_user_req::CreateUserReq;
use crate::services::friend_request_service::FriendRequestService;
use crate::services::mongo_service::MongoService;

use crate::services::tcp::{self, send_to_tcp_server};
use crate::services::user_service::UserService;
use crate::services::wallet_service::WalletService;
use crate::utils::auth::{Claims, decode_token, encode_token};
use crate::utils::formatter;
use crate::utils::send_email::send_email;


#[get("/say_hello")]
pub async fn say_hello(claim:Option<ReqData<Claims>>)-> HttpResponse{
    print!("Hello maaa gee");
    if let Some(claim) = claim{
        print!("{:?}", claim)
    }

    print!("Hello maaa gee");
    let response = Response{
        message:"good".to_string(),
    };
    // match DB::initialize_db().await{
    //     Ok((_))=>{},
    //     Err(err)=>{println!("{:?}", err)}
    // }
    // let tdata = decode_token("\
    // eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiUGF0aWVudCIsImVtYWlsIjoicGF0aWVudDFAeC5jb20iLCJuYW1lIjoicGF0aWVudDEiLCJleHAiOjE2NzU1NjI1ODB9.lSUV9_cvLqYXgsvfvbbr5s_QqDtFzbIux6ePVSKu9xo\
    // ".to_string());
    // let tdata = match tdata {
    //     Ok(tdata)=>{tdata},
    //     Err(err)=>{return HttpResponse::InternalServerError().json(err.to_string())}
    // };
    // println!("{:?}",  tdata);
    return HttpResponse::Created().json(response)
}

#[post("/user")]
pub async fn create_user(database:Data<MongoService>, new_user:Json<CreateUserReq>)->HttpResponse{
    println!("new req");
    let user = User{
        user_name:new_user.user_name.to_owned(),
        created_at:chrono::offset::Utc::now().to_string(),
        email:new_user.email.to_owned(),
        code:Option::from(93030),
        user_type: new_user.into_inner().user_type,
        id:None
    };

    // check if user exists
    if check_if_user_exists(&database, &user.email).await {
        return HttpResponse::BadRequest()
            .json(Response{message:"This user exists".to_string()})
    }
    // setup player data if the user is a player
    if user.user_type == UserType::User{
        let user_res = UserService::create_user(database.db.borrow(),&user).await;

        match user_res {
            Ok(user)=> return HttpResponse::Ok().json(user),
            Err(err)=>return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    }

    return HttpResponse::Ok().json(Response{message:"Successfully created".to_string()})

}




#[post("/user/login")]
pub async fn login_user(database:Data<MongoService>, req_data:Json<LoginReq>)->HttpResponse{
    //validate request data
    {
        match req_data.borrow().validate() {
            Ok(_)=>{},
            Err(err)=>{
                return HttpResponse::BadRequest().json(err);
            }
        }
    }

    // convert code to int
    let code = req_data.code.parse::<i32>();
    let code = match code {
        Ok(code)=>{code},
        Err(err)=>{return return HttpResponse::BadRequest().
            json(Response{message:"Error  getting string".to_string()})}
    };

    // check if the user sent the right otp
    // get user data from db
    let get_user_res = UserService::get_by_email(
        database.db.borrow(), req_data.borrow().email.to_owned()).await;
    let user = match  get_user_res{
        Ok(user)=>{
            match user {
                Some(user)=>{user},
                None=>{return return HttpResponse::InternalServerError().
                    json(Response{message:"User Not Found".to_string()})}
            }
        },
        Err(err)=>{
            // log error
            return HttpResponse::InternalServerError().
            json(Response{message:"Error getting user".to_string()})}
    };
    let real_code = match user.code{
        Some(real_code)=>{real_code},
        None=>{
            return HttpResponse::BadRequest().
                json(Response{message:"Get auth code".to_string()})
        }
    };
    //check if user has the right code
    if (real_code !=  code){
        return HttpResponse::Unauthorized().
            json(Response{message:"Wrong auth data".to_string()})
    }
    //if he has the right code send email
    {
        send_new_login_email(user.borrow(), chrono::offset::Utc::now().to_string());
    }

    // send token
    let login_token =encode_token(
        user.user_type, req_data.borrow().email.as_str().to_string(),user.user_name);
    let login_token = match login_token {
        Ok(login_token)=>{login_token},
        Err(err)=>{
            return HttpResponse::InternalServerError().
                json(Response{message:"Error getting token".to_string()})
        }
    };

    HttpResponse::Ok()
        .json(LoginResp{message:"Logged in".to_string(), token:login_token})
}


#[post("/user/get_code")]
pub async fn get_code(database:Data<MongoService>, req_data:Json<GetCodeReq>)->HttpResponse {
    //validate request data
    {
        match req_data.borrow().validate() {
            Ok(_) => {},
            Err(err) => {
                return HttpResponse::BadRequest().json(err);
            }
        }
    }

    // generate code
    let mut rand_rng = rand::thread_rng();
    let code = rand_rng.gen_range(0..999999);
    // get user profile
    let user = UserService::get_by_email(&database.db, req_data.email.to_string()).await;
    let mut user = match user {
        Ok(user)=>{
            match user {
                Some(user)=>{user},
                None=>{
                    return HttpResponse::BadRequest()
                        .json(Response{message:"User not found".to_string()})
                }
            }
        },
        Err(err)=>{return HttpResponse::InternalServerError()
            .json(Response{message:err.to_string()})}
    };

    // update user data with new code
    user.code = Option::from(code);
    let update_res = UserService::update(&database.db, &user.email, &user).await;
    let update_res =match update_res{
        Ok(update_res)=>{update_res},
        Err(err)=>{
            return HttpResponse::InternalServerError()
                .json(Response{message:err.to_string()})
        }
    };
    //
    HttpResponse::Ok()
        .json(CodeResp{code: code})
}

async fn send_new_login_email(user:&User, time:String){
    let name = user.user_name.as_str().to_string();

    let mut reg = Handlebars::new();
    let order_email_content = reg.render_template (
        include_str!("../utils/html/new_login.hbs"),
        &serde_json::json!({"name" :name, "time":time})).unwrap();

    let email_data = EmailData{
        subject:"New Login".to_string(),
        to: (*user.email).parse().unwrap(),
        body: order_email_content
    };
    send_email(email_data);
}


// check if user exists
pub async fn check_if_user_exists(database:&Data<MongoService>, email:&String)->bool {
    let mut ok = false;

    let user = UserService::get_by_email(
        &database.db,
        email.to_string()
    ).await;

    match user {
        Ok(user)=>{
            match user {
                Some(_)=>{ok=true},
                None=>{ok=false}
            }
        },
        Err(_)=>{ok = false}
    }

    return ok;
}

// check if user exists
pub async fn check_if_user_exists_username(database:&Data<MongoService>, user_name:&String)->bool {
    let mut ok = false;

    let user = UserService::get_by_(
        &database.db,
        doc! {"user_name":user_name}
    ).await;

    match user {
        Ok(user)=>{
            match user {
                Some(_)=>{ok=true},
                None=>{ok=false}
            }
        },
        Err(_)=>{ok = false}
    }

    return ok;
}

#[post("/user/kura_login")]
pub async fn kura_id_login(database:Data<MongoService>, req_data:Json<CreateKuracoinID>)->HttpResponse {
    let mut respData = GenericResp::<String>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
   
    //validate request data
    
    {
        match req_data.borrow().validate() {
            Ok(_) => {},
            Err(err) => {
                log::error!(" validation error  {}", err.to_string());
                respData.message = "Error validating request data".to_string();
                respData.server_message =Some(err.to_string());
                respData.data = None;
                return HttpResponse::BadRequest().json(respData);
            }
        }
    }


    // check if user exist
    if !check_if_user_exists_username(&database, &req_data.user_name.to_owned()).await{
        println!("{}", "User does not exists");
        respData.message = "User does not exists".to_string();
        respData.server_message =None;
        respData.data = None;
        return HttpResponse::BadRequest().json(respData);
    }

    
 

    // send message to the kuracoin blockchain to create new user
    let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            log::error!(" error getting vhenncoin server id {}", err.to_string());
            respData.message = "Error connecting to blockchain".to_string();
            respData.server_message =Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };

    let message_data = match serde_json::to_string(&req_data){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error persing req data  {}", err.to_string());
            respData.message = "Error persing data".to_string();
            respData.server_message =Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);  
        }
    };
    let message = formatter::Formatter::request_formatter(
        "ValidateUserId".to_string(), 
        message_data,
        "".to_string(), 
        "".to_string(),
        "0".to_string());

    let m = message.clone();
    let ip = kura_coin_server_ip.clone();
    let result = web::block(move || send_to_tcp_server(m,ip  )).await;
    let response_string =match result {
        Ok(data)=>{
            match data {
                Ok(data)=>{data},
                Err(err)=>{
                    log::info!(" error from blockchain {}", err.to_string());
                    respData.message = "Error persing data".to_string();
                    respData.server_message =Some(err.to_string());
                    respData.data = None;
                    return HttpResponse::BadRequest().json(respData);     
                }
            }
        },
        Err(err)=>{ 
            log::error!(" error from blockchain tcpreq  {}", err.to_string());
            respData.message = "Error persing data".to_string();
            respData.server_message =Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);   
        }
    };

    let resp_data: Vec<&str>= response_string.split('\n').collect();
    let code = match resp_data.get(0){
        Some(data)=>{data},
        None=>{
            respData.message = "Error with blockchain response data".to_string();
            respData.server_message =None;
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);     
        }
    };

    if(*code != "1"){
        // blockchain request failed
        respData.message = "Failed to create digital ID on blockchain".to_string();
            respData.server_message =None;
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);     
    }


    // get user data from db after successful login
    let user = UserService::get_by_(
        &database.db,
        doc! {"user_name":req_data.user_name.to_owned()}
    ).await;

    let user = match user {
        Ok(user)=>{
            match user {
                Some(data)=>{data},
                None=>{
                    respData.message = "Could not find user".to_string();
                    respData.server_message =None;
                    respData.data = None;
                    return HttpResponse::BadRequest().json(respData);
                }
            }
        },
        Err(err)=>{
            log::error!(" error getting user  {}", err.to_string());
            respData.message = "Error getting user data".to_string();
            respData.server_message =None;
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };


    // generate token for user
    let login_token =encode_token(
        user.user_type, user.email,user.user_name);

    let login_token = match login_token {
        Ok(login_token)=>{login_token},
        Err(err)=>{
            log::error!(" error making login token {}", err.to_string());
            return HttpResponse::InternalServerError().
                json(Response{message:"Error getting token".to_string()})
        }
    };


    respData.message = "Ok".to_string();
    respData.server_message =None;
    respData.data = Some(login_token);
    return HttpResponse::Ok().json(respData);


}


#[post("/user/kura_signup")]
pub async fn kura_id_signup(database:Data<MongoService>, req_data:Json<CreateKuracoinID>)->HttpResponse {
    let mut respData = GenericResp::<String>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: None
    };
   
    //validate request data
    
    {
        match req_data.borrow().validate() {
            Ok(_) => {},
            Err(err) => {
                log::error!(" validation error {}", err.to_string());
                respData.message = "Error validating request data".to_string();
                respData.server_message =Some(err.to_string());
                respData.data = None;
                return HttpResponse::BadRequest().json(respData);
            }
        }
    }


    // check if user exist
    if check_if_user_exists_username(&database, &req_data.user_name.to_owned()).await{
        log::info!(" User exists ");
        respData.message = "User already exists".to_string();
        respData.server_message =None;
        respData.data = None;
        return HttpResponse::BadRequest().json(respData);
    }

    
 

    // send message to the kuracoin blockchain to create new user
    let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
        Ok(data)=>{data.to_owned()},
        Err(err)=>{
            log::error!(" error getting vhenncoin server IP  {}", err.to_string());
            respData.message = "Error connecting to blockchain".to_string();
            respData.server_message =Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);
        }
    };

    let message_data = match serde_json::to_string(&req_data){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error persing request data   {}", err.to_string());
            respData.message = "Error persing data".to_string();
            respData.server_message =Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);  
        }
    };
    let message = formatter::Formatter::request_formatter(
        "CreateUserId".to_string(), 
        message_data,
        "".to_string(), 
        "".to_string(),
        "0".to_string());

    let m = message.clone();
    let ip = kura_coin_server_ip.clone();
    let result = web::block(move || send_to_tcp_server(m,ip  )).await;
    let response_string =match result {
        Ok(data)=>{
            match data {
                Ok(data)=>{data},
                Err(err)=>{
                    log::info!(" error from tcp {}", err.to_string());
                    respData.message = "Error from blockchain".to_string();
                    respData.server_message =Some(err.to_string());
                    respData.data = None;
                    return HttpResponse::BadRequest().json(respData);     
                }
            }
        },
        Err(err)=>{ 
            log::error!(" error from blockchain tcp req {}", err.to_string());
            respData.message = "Error persing data".to_string();
            respData.server_message =Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);   
        }
    };

    let resp_data: Vec<&str>= response_string.split('\n').collect();
    let code = match resp_data.get(0){
        Some(data)=>{data},
        None=>{
            respData.message = "Error with blockchain response data".to_string();
            respData.server_message =None;
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);     
        }
    };

    if(*code != "1"){
        // blockchain request failed
        respData.message = "Failed to create digital ID on blockchain".to_string();
            respData.server_message =None;
            respData.data = None;
            return HttpResponse::BadRequest().json(respData);     
    }




    let user = User{
        user_name:req_data.user_name.to_owned(),
        created_at:chrono::offset::Utc::now().to_string(),
        email:"".to_string(),
        code:Option::from(93030),
        user_type:UserType::User,
        id:None
    };

  
    // setup
    if user.user_type == UserType::User{
        let user_res = UserService::create_user(database.db.borrow(),&user).await;

        match user_res {
            Ok(user)=> {},
            Err(err)=>{
                log::error!(" error creating user {}", err.to_string());
                respData.message = "Error creating user".to_string();
                respData.server_message =None;
                respData.data = None;
                return HttpResponse::BadRequest().json(respData);  
            }
        }
    }


    // 

    respData.message = "Ok".to_string();
    respData.server_message =None;
    respData.data = Some(req_data.user_name.to_owned());
    return HttpResponse::Ok().json(respData);
}

#[post("/friend_request/send")]
pub async fn send_friend_request(
    database:Data<MongoService>,
     req: Result<web::Json<SendFriendReq>, actix_web::Error>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<FriendRequest>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(FriendRequest::default())
    };
    let req = match req {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::InternalServerError().json( respData); 
        }
    };

    match UserService::get_by_(&database.db, doc! {"user_name":req.user_name.clone()}).await{
        Ok(data)=>{
            match data {
                Some(_)=>{},
                None=>{
                    respData.message = "User not found".to_string();
                    respData.server_message =None;
                    respData.data = None;
                    return HttpResponse::BadRequest().json(respData);   
                }
            }
        },
        Err(err)=>{
            respData.message = "Error getting username".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::BadRequest().json(respData);
        }
    }

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    let friend_request = FriendRequest{
        id: uuid::Uuid::new_v4().to_string(),
        user_name: req.user_name.clone(),
        requester:claim.user_name.clone(),
        status:FriendRequestStatus::PENDING,
        created_at:chrono::offset::Utc::now().to_string(),
        updated_at:chrono::offset::Utc::now().to_string(),
        requester_profile: None
    };

    match FriendRequestService::create_friend_request(&database.db, friend_request).await{
        Ok(data)=>{data}, 
        Err(err)=>{
            respData.message = "Error creating friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData);
        }
    };


    return HttpResponse::Ok().json({});

}


#[derive(Debug, Deserialize)]
struct GenID{id:String} 
#[get("/friend_request/accept/{id}")]
pub async fn accept_friend_request(
    database:Data<MongoService>,
    path: web::Path<GenID>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<FriendRequest>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(FriendRequest::default())
    };

    // get friend request 
    let fr = match FriendRequestService::get_single_friend_request(&database.db, path.id.clone()).await{
        Ok(data)=>{
            match data{
                Some(data)=>{data},
                None=>{
                    respData.message = "Error finding request".to_string();
                    respData.server_message = None;
                    respData.data = None;
                    return HttpResponse::BadRequest().json(respData);   
                }
            }
        },
        Err(err)=>{
            respData.message = "Error accepting friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData);
        }
    };


    
    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // check if the req owner owns the friend request
    if fr.user_name != claim.user_name.clone(){
        respData.message = "you cannot accept this request".to_string();
        respData.server_message = None;
        respData.data = None;

        return HttpResponse::BadRequest().json(respData);
    }
 
    match FriendRequestService::accept_friend_request(&database.db, fr).await{
        Ok(data)=>{data}, 
        Err(err)=>{
            respData.message = "Error accepting friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData);
        }
    };


    return HttpResponse::Ok().json({});

}



#[get("/friend_requests")]
pub async fn get_my_friend_request(
    database:Data<MongoService>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Vec<FriendRequest>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(vec![FriendRequest::default()])
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(
                    respData
                )
        }
    };

    // get friend request 
    let fr = match FriendRequestService::get_user_friend_request(&database.db, claim.user_name.clone()).await{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Error getting friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            log::error!("error getting friend requests  {}", err.to_string());
            return HttpResponse::InternalServerError().json(respData);
        }
    };
    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(fr);

    return HttpResponse::Ok().json(respData);

}
