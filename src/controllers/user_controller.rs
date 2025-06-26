use std::borrow::{Borrow, BorrowMut};
use std::env;
use std::future::IntoFuture;
use actix_web::{Responder, get, HttpResponse, web::Json, post};

use actix_web::web::{self, Data, ReqData};
use bcrypt::{hash, verify, DEFAULT_COST};
use bigdecimal::{BigDecimal, Zero};
use handlebars::Handlebars;
use log::error;
use mongodb::bson::doc;
use rand::Rng;
use regex::Replacer;
use serde::Deserialize;
use serde_derive::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;
use crate::controllers::service_errors::ServiceError;
use crate::database::db::db::DB;
use crate::models::fried_request::{FriendRequest, FriendRequestStatus};
use crate::models::helper::EmailData;
use crate::models::power_up::{PlayerPowerUp, PowerUpType};
use crate::models::profile::Profile;
use crate::models::request_models::{ConfirmAccountReq, CreateKuracoinID, GetCodeReq, LoginReq, ResendCodeReq, SendFriendReq};
use crate::models::response::{CodeResp, GenericResp, LoginResp, PlayerRunInfoRes, Response};
use crate::models::run_info::RunInfo;
use crate::models::user::{User};
use crate::models::wallet::Wallet;
use crate::req_models::create_user_req::{CreateUserReq};
use crate::req_models::requests::{ChangePasswordReq, GetPasswordResetCodeReq, UpdateProfileReq};
use crate::services::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};
use crate::services::email_service::EmailService;
use crate::services::friend_request_service::{FriendRequestService, FriendRequestWithProfile};
use crate::services::mongo_service::MongoService;

use crate::services::profile_service::{MiniProfile, ProfileService};
use crate::services::system_service::SystemService;
use crate::services::tcp::{self, send_to_tcp_server};
use crate::services::user_service::UserService;
use crate::services::wallet_service::WalletService;
use crate::utils::auth::{Claims, decode_token, encode_token};
use crate::utils::formatter;
use crate::utils::general::{get_time_naive, has_no_spaces, is_all_lowercase};
// use crate::utils::send_email::send_email;


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

// #[post("/user")]
// pub async fn create_user(database:Data<MongoService>, new_user:Json<CreateUserReq>)->HttpResponse{
//     println!("new req");
//     let user = User{
//         user_name:new_user.user_name.to_owned(),
//         created_at:chrono::offset::Utc::now().to_string(),
//         email:new_user.email.to_owned(),
//         code:Option::from(93030),
//         user_type: new_user.into_inner().user_type,
//         id:None
//     };

//     // check if user exists
//     if check_if_user_exists(&database, &user.email).await {
//         return HttpResponse::BadRequest()
//             .json(Response{message:"This user exists".to_string()})
//     }
//     // setup player data if the user is a player
//     if user.user_type == UserType::User{
//         let user_res = UserService::create_user(database.db.borrow(),&user).await;

//         match user_res {
//             Ok(user)=> return HttpResponse::Ok().json(user),
//             Err(err)=>return HttpResponse::InternalServerError()
//                 .json(Response{message:err.to_string()})
//         }
//     }

//     return HttpResponse::Ok().json(Response{message:"Successfully created".to_string()})

// }

#[post("/create_account")]
pub async fn create_account(
    pool: web::Data<PgPool>,
    new_user:Result<Json<CreateUserReq>, actix_web::Error>
)->HttpResponse{
    println!("new req");
    let mut resp_data = GenericResp::<String>{
        message: "".to_string(),
        server_message: None,
        data: None
    };

    let new_user = match new_user {
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("validation  error  {}", err.to_string());
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError().json( resp_data);
        }
    };
    let hashed_password = hash(new_user.password.clone(), DEFAULT_COST).unwrap();
    // make sure user name is lowercase 
    if !is_all_lowercase(new_user.user_name.clone().as_str()){
        resp_data.message = "Username should be lowercased".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest()
            .json(resp_data)  
    }
    
    if !has_no_spaces(new_user.user_name.clone().as_str()){
        resp_data.message = "Username should have no spaces".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest()
            .json(resp_data)
    }
    match ProfileService::user_exists(&pool, &*new_user.user_name.clone()).await{
        Ok(user) => {
            if(user){
                resp_data.message = "User already exists".to_string();
                resp_data.server_message =None;
                resp_data.data = None;
                return HttpResponse::BadRequest()
                    .json(resp_data)
            }
        },
        Err(e) => {
            log::error!("error getting user data {}", e );
            resp_data.message = "Error getting user".to_string();
            resp_data.server_message = Option::from(e.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError()
                .json(resp_data)
        }
    }


    
    let user = User{
        user_name:new_user.user_name.to_owned(),
        created_at:get_time_naive(),
        email:Some(new_user.email.to_owned()),
        code:None,
        user_type: "USER".to_string(),
        id: Uuid::new_v4().to_string(),
        password_hash: hashed_password,
        is_deleted:false,
        email_confirmed: false
    };

    // let code: String = rand::thread_rng()
    //     .gen_range(100_000..1_000_000) // Generates a 6-digit number
    //     .to_string();
    // match EmailService::send_signup_email(new_user.email.clone(), code ).await{
    //     Ok(email)=>{},
    //     Err(err)=>{
    //         resp_data.message = "Error sending email".to_string();
    //         resp_data.server_message = Some(err.to_string());
    //         resp_data.data = None;
    //         return HttpResponse::InternalServerError().json(resp_data)
    //     }
    // };

    
    // setup player data if the user is a player
    let code = rand::thread_rng()
        .gen_range(100_000..1_000_000) ;
    if user.user_type == "USER"{
        let user_res = UserService::create_user(&pool,user.clone(), code.clone()).await;

        match user_res {
            Ok(_)=> {
            },
            Err(err)=>{

                resp_data.message = "Error creating user".to_string();
                if err.to_string() == "USER_EXISTS"{
                    resp_data.message = "Username already exists".to_string();
                }
                resp_data.server_message = Some(err.to_string());
                resp_data.data = None;
                return HttpResponse::InternalServerError()
                .json(resp_data)
            }
        }


        // send email verification

    }else{
        resp_data.message = "Wrong user type".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest().json(resp_data)
    }
    let email_service = &EmailService::new();
    match EmailService::send_signup_email2(email_service, user.email.unwrap(),code.to_string() ).await{
        Ok(_) => {},
        Err(err)=>{
            log::error!("Failed to send signup email: {}", err);
        }
    };

    // if there is a referal then update
    if new_user.referral.is_some(){
        let sys_data = match SystemService::get_system_data(&pool).await{
            Ok(sys_data)=>sys_data,
            Err(err)=>{
                error!("{}", err);
                None
            }
        };
       let ref_amount = match sys_data{
           Some(d)=>{ d.ref_amount},
           None=>{BigDecimal::zero()}
       };
        
        // get profile of new user 
        let new_user_profile = match ProfileService::get_profile(&pool, new_user.user_name.clone()).await{
            Ok(data)=>{Some(data)},
            Err(err)=>{
                error!("{}", err);
                None
            }
        };
        
        
        // get the user and update his referrals
        let ref_profile = match ProfileService::get_profile(&pool, new_user.referral.to_owned().unwrap()).await{
            Ok(profile)=>{Some(profile)},
            Err(err)=>{
               None 
            }
        };
        
        // update referal user if he does not already have this user and forget result
        if ref_profile.is_some(){
          let mut n_ref_profile = ref_profile.clone().unwrap();  
            if !n_ref_profile.referred_users.contains(&new_user.user_name.to_owned()){
                n_ref_profile.referred_users.push(new_user.user_name.to_owned());
                // ref payment 
                n_ref_profile.unclaimed_earnings = ref_profile.unwrap().clone().unclaimed_earnings + ref_amount.to_owned();
                ProfileService::update_profile(&pool, n_ref_profile).await;
                // update the newly registered user unclaimed money
                if new_user_profile.is_some(){
                    let mut new_up = new_user_profile.clone().unwrap();
                    new_up.unclaimed_earnings = ref_amount/2;
                    ProfileService::update_profile(&pool, new_up ).await;  
                }
            }
        }
    }
    
    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = None;
    HttpResponse::Ok().json(resp_data)
}
#[post("/resend_code")]
pub async fn resend_code(pool:Data<PgPool>, req:Json<ResendCodeReq>)->HttpResponse {
    let mut resp_data = GenericResp::<String>{
        message: "".to_string(),
        server_message: None,
        data: None
    };
    // validate data
    match req.validate(){
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::BadRequest().json(resp_data)
        }
    };

    match UserService::update_code(&pool, req.email.clone()).await{
        Ok(_)=>{
            resp_data.message = "Ok".to_string();
            return HttpResponse::Ok().json(resp_data)
        }
        Err(err)=>{
            log::error!("error confirming user ct {}", err);
            resp_data.message = err.to_string();
            return HttpResponse::BadRequest().json(resp_data)
        },
    }
}

#[post("/get_reset_password_code")]
pub async fn get_reset_password_code(
    pool:Data<PgPool>,
    req: Result<web::Json<GetPasswordResetCodeReq>, actix_web::Error>,
)->HttpResponse {
    let mut respData = GenericResp::<String>{
        message: "".to_string(),
        server_message: None,
        data: None
    };
    // validate data
    let req= match req{
        Ok(data)=>{data},
        Err(err)=>{
            respData.message = "Validation error".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            return HttpResponse::BadRequest().json(respData)
        }
    };

    // get user account
    let user =match UserService::get_by_username(&pool,req.user_name.clone() ).await{
        Ok(data)=>{match data{
            Some(user)=>{user},
            None=>{
                respData.message = "User account not found".to_string();
                respData.server_message = None;
                respData.data = None;
                return HttpResponse::BadRequest().json(respData)
            }
        }},
        Err(err)=>{
            log::error!("error getting user {}", err);
            respData.message = "Error getting user".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    // derive code
    let code = rand::thread_rng()
        .gen_range(100_000..1_000_000) ;
    //update user
    let mut new_user = user.clone();
    new_user.code = Some(code);
    match UserService::update(&pool, new_user.clone()).await{
        Ok(_)=>{}
        Err(err)=>{
            log::error!("error updating user {}", err);
            respData.message = "Error updating user".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    // send email
    let email_service = &EmailService::new();
    match EmailService::send_reset_password_email2(email_service,user.email.unwrap_or_default(),code.to_string() ).await{
        Ok(_)=>{},
        Err(err)=>{
            log::error!("error sending reset password email {}", err);
            respData.message = "Error sending email".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = None;
    return HttpResponse::Ok().json(respData);
}

#[post("/confirm_account")]
pub async fn confirm_account(pool:Data<PgPool>, req:Json<ConfirmAccountReq>)->HttpResponse {
    let mut resp_data = GenericResp::<String>{
        message: "".to_string(),
        server_message: None,
        data: None
    };
    // validate data
    match req.validate(){
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::BadRequest().json(resp_data)
        }
    };
    
    match UserService::confirm_user_email(&pool, req.email.clone(), req.code.clone()).await{
        Ok(_)=>{
            resp_data.message = "Ok".to_string();
            return HttpResponse::Ok().json(resp_data)
        }
        Err(err)=>{
            log::error!("error confirming user ct {}", err);
            resp_data.message = err.to_string();
            return HttpResponse::BadRequest().json(resp_data)
        },
    }
}


#[post("/change_password")]
pub async fn change_password(
    pool:Data<PgPool>,
    req: Result<web::Json<ChangePasswordReq>, actix_web::Error>,
)->HttpResponse {
    let mut respData = GenericResp::<String> {
        message: "".to_string(),
        server_message: None,
        data: None
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

    // get user account
    let user =match UserService::get_by_username(&pool, req.user_name.clone()).await{
        Ok(data)=>{match data{
            Some(user)=>{user},
            None=>{
                respData.message = "User account not found".to_string();
                respData.server_message = None;
                respData.data = None;
                return HttpResponse::BadRequest().json(respData)
            }
        }},
        Err(err)=>{
            log::error!("error getting user {}", err);
            respData.message = "Error getting user".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };
    // check code
    if req.code != user.code.unwrap_or_default().to_string(){
        respData.message = "Invalid code".to_string();
        respData.server_message = None;
        respData.data = None;
        return HttpResponse::BadRequest().json(respData)
    }
    // update password
    let hashed_password = match hash(req.password.clone(), DEFAULT_COST){
        Ok(hashed)=>{hashed},
        Err(err)=>{
            log::error!("error encrypting password");
            respData.message = "Password error".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::BadRequest().json(respData)
        }
    };
    let mut new_user = user.clone();
    new_user.password_hash = hashed_password;
    match UserService::update(&pool, new_user).await{
        Ok(_)=>{}
        Err(err)=>{
            log::error!("error updating user {}", err);
            respData.message = "Error updating user".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData)
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = None;
    return HttpResponse::Ok().json(respData)
}


#[post("/login")]
pub async fn login(pool:Data<PgPool>, req:Json<LoginReq>)->HttpResponse{
    #[derive(Debug, Serialize, Deserialize)]
    struct LoginResp{
        pub token:String,
        pub email_confirmed:bool,
        pub email:String
    }
    let mut resp_data = GenericResp::<LoginResp>{
        message: "".to_string(),
        server_message: None,
        data: None
    };

    // validate data
    match req.validate(){
        Ok(data)=>{data},
        Err(err)=>{
            resp_data.message = "Validation error".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
        }
    };

    // check if user exists
    let user_res = UserService::get_by_username(
        &pool, req.user_name.clone()).await;

    let  user = match user_res {
        Ok(user)=>{match user{
            Some(data)=>data,
            None=>{
                resp_data.message = "Error getting user".to_string();
                resp_data.server_message = None;
                resp_data.data = None;
                return HttpResponse::BadRequest()
                    .json(resp_data) 
            }
        }},
        Err(err)=>{
            resp_data.message = "Error getting user".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError()
            .json(resp_data)
        }
    };

    // check if user has been deleted
    if user.is_deleted {
        resp_data.message = "This account is deleted".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest()
            .json(resp_data) 
    }

    
    let hashed_password = hash(req.password.clone(), DEFAULT_COST).unwrap();

    //log::debug!("HASH {} --- real hash {}", hashed_password, user.password_hash);
    let is_valid = match verify(req.password.clone(), &user.password_hash){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!("verify password error .. {}", err.to_string());
            resp_data.message = "verification data".to_string();
            resp_data.server_message = Some(err.to_string());
            resp_data.data = None;
            return HttpResponse::InternalServerError()
            .json(resp_data) 

        }
    };
    // compare passwords
    if !is_valid{
        resp_data.message = "Invalid login data".to_string();
        resp_data.server_message = None;
        resp_data.data = None;
        return HttpResponse::BadRequest()
        .json(resp_data) 
    }

    // make token

    let login_token =encode_token(
        user.user_type,"".to_string(),user.user_name);
    let login_token = match login_token {
        Ok(login_token)=>{login_token},
        Err(err)=>{
            return HttpResponse::InternalServerError().
                json(Response{message:"Error getting token".to_string()})
        }
    };
    
  
    resp_data.message = "Ok".to_string();
    resp_data.server_message = None;
    resp_data.data = Some(LoginResp{
        token: login_token,
        email_confirmed: user.email_confirmed,
        email: user.email.unwrap_or_default()
    });
    return HttpResponse::Ok().json(resp_data)

}



// #[post("/user/login")]
// pub async fn login_user(database:Data<MongoService>, req_data:Json<LoginReq>)->HttpResponse{
//     //validate request data
//     {
//         match req_data.borrow().validate() {
//             Ok(_)=>{},
//             Err(err)=>{
//                 return HttpResponse::BadRequest().json(err);
//             }
//         }
//     }

//     // convert code to int
//     let code = req_data.code.parse::<i32>();
//     let code = match code {
//         Ok(code)=>{code},
//         Err(err)=>{return return HttpResponse::BadRequest().
//             json(Response{message:"Error  getting string".to_string()})}
//     };

//     // check if the user sent the right otp
//     // get user data from db
//     let get_user_res = UserService::get_by_email(
//         database.db.borrow(), req_data.borrow().email.to_owned()).await;
//     let user = match  get_user_res{
//         Ok(user)=>{
//             match user {
//                 Some(user)=>{user},
//                 None=>{return return HttpResponse::InternalServerError().
//                     json(Response{message:"User Not Found".to_string()})}
//             }
//         },
//         Err(err)=>{
//             // log error
//             return HttpResponse::InternalServerError().
//             json(Response{message:"Error getting user".to_string()})}
//     };
//     let real_code = match user.code{
//         Some(real_code)=>{real_code},
//         None=>{
//             return HttpResponse::BadRequest().
//                 json(Response{message:"Get auth code".to_string()})
//         }
//     };
//     //check if user has the right code
//     if (real_code !=  code){
//         return HttpResponse::Unauthorized().
//             json(Response{message:"Wrong auth data".to_string()})
//     }
//     //if he has the right code send email
//     {
//         send_new_login_email(user.borrow(), chrono::offset::Utc::now().to_string());
//     }

//     // send token
//     let login_token =encode_token(
//         user.user_type, req_data.borrow().email.as_str().to_string(),user.user_name);
//     let login_token = match login_token {
//         Ok(login_token)=>{login_token},
//         Err(err)=>{
//             return HttpResponse::InternalServerError().
//                 json(Response{message:"Error getting token".to_string()})
//         }
//     };

//     HttpResponse::Ok()
//         .json(LoginResp{message:"Logged in".to_string(), token:login_token})
// }


// #[post("/user/get_code")]
// pub async fn get_code(database:Data<MongoService>, req_data:Json<GetCodeReq>)->HttpResponse {
//     //validate request data
//     {
//         match req_data.borrow().validate() {
//             Ok(_) => {},
//             Err(err) => {
//                 return HttpResponse::BadRequest().json(err);
//             }
//         }
//     }

//     // generate code
//     let mut rand_rng = rand::thread_rng();
//     let code = rand_rng.gen_range(0..999999);
//     // get user profile
//     let user = UserService::get_by_email(&database.db, req_data.email.to_string()).await;
//     let mut user = match user {
//         Ok(user)=>{
//             match user {
//                 Some(user)=>{user},
//                 None=>{
//                     return HttpResponse::BadRequest()
//                         .json(Response{message:"User not found".to_string()})
//                 }
//             }
//         },
//         Err(err)=>{return HttpResponse::InternalServerError()
//             .json(Response{message:err.to_string()})}
//     };

//     // update user data with new code
//     user.code = Option::from(code);
//     let update_res = UserService::update(&database.db, &user.email, &user).await;
//     let update_res =match update_res{
//         Ok(update_res)=>{update_res},
//         Err(err)=>{
//             return HttpResponse::InternalServerError()
//                 .json(Response{message:err.to_string()})
//         }
//     };
//     //
//     HttpResponse::Ok()
//         .json(CodeResp{code: code})
// }

// async fn send_new_login_email(user:&User, time:String){
//     let name = user.user_name.as_str().to_string();

//     let mut reg = Handlebars::new();
//     let order_email_content = reg.render_template (
//         include_str!("../utils/html/new_login.hbs"),
//         &serde_json::json!({"name" :name, "time":time})).unwrap();

//     let email_data = EmailData{
//         subject:"New Login".to_string(),
//         to: (*user.email).parse().unwrap(),
//         body: order_email_content
//     };
//     send_email(email_data);
// }


// check if user exists
pub async fn check_if_user_exists(pool:&Data<PgPool>, email:&String)->bool {
    let mut ok = false;

    let user = UserService::get_by_username(
        &pool,
        email.to_string()
    ).await;

    match user {
        Ok(user)=>{
            ok = true;
        },
        Err(_)=>{ok = false}
    }

    return ok;
}



// check if user exists
// pub async fn check_if_user_exists_user_name(pool:&Data<DbPool>, user_name:&String)->bool {
//     let mut ok = false;
// 
//     let user = UserService::get_by_username(
//         &pool,
//         user_name
//     ).await;
// 
//     match user {
//         Ok(user)=>{
//          ok = true;
//         },
//         Err(_)=>{ok = false}
//     }
// 
//     return ok;
// }



// #[post("/user/kura_login")]
// pub async fn kura_id_login(database:Data<MongoService>, req_data:Json<CreateKuracoinID>)->HttpResponse {
//     let mut respData = GenericResp::<String>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: None
//     };
   
//     //validate request data
    
//     {
//         match req_data.borrow().validate() {
//             Ok(_) => {},
//             Err(err) => {
//                 log::error!(" validation error  {}", err.to_string());
//                 respData.message = "Error validating request data".to_string();
//                 respData.server_message =Some(err.to_string());
//                 respData.data = None;
//                 return HttpResponse::BadRequest().json(respData);
//             }
//         }
//     }


//     // check if user exist
//     if !check_if_user_exists_username(&database, &req_data.user_name.to_owned()).await{
//         println!("{}", "User does not exists");
//         respData.message = "User does not exists".to_string();
//         respData.server_message =None;
//         respData.data = None;
//         return HttpResponse::BadRequest().json(respData);
//     }

    
 

//     // send message to the kuracoin blockchain to create new user
//     let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
//         Ok(data)=>{data.to_owned()},
//         Err(err)=>{
//             log::error!(" error getting vhenncoin server id {}", err.to_string());
//             respData.message = "Error connecting to blockchain".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };

//     let message_data = match serde_json::to_string(&req_data){
//         Ok(data)=>{data},
//         Err(err)=>{
//             log::error!(" error persing req data  {}", err.to_string());
//             respData.message = "Block chain server error".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);  
//         }
//     };
//     let message = formatter::Formatter::request_formatter(
//         "ValidateUserId".to_string(), 
//         message_data,
//         "".to_string(), 
//         "".to_string(),
//         "0".to_string());

//     let m = message.clone();
//     let ip = kura_coin_server_ip.clone();
//     let result = web::block(move || send_to_tcp_server(m,ip  )).await;
//     let response_string =match result {
//         Ok(data)=>{
//             match data {
//                 Ok(data)=>{data},
//                 Err(err)=>{
//                     log::info!(" error from blockchain {}", err.to_string());
//                     respData.message = "Error from blockchain server".to_string();
//                     respData.server_message =Some(err.to_string());
//                     respData.data = None;
//                     return HttpResponse::BadRequest().json(respData);     
//                 }
//             }
//         },
//         Err(err)=>{ 
//             log::error!(" error from blockchain tcpreq  {}", err.to_string());
//             respData.message = "Error from blockchian server".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);   
//         }
//     };

//     let resp_data: Vec<&str>= response_string.split('\n').collect();
//     let code = match resp_data.get(0){
//         Some(data)=>{data},
//         None=>{
//             respData.message = "Error with blockchain response data".to_string();
//             respData.server_message =None;
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);     
//         }
//     };
//     let mess = match resp_data.get(1){
//         Some(data)=>{data},
//         None=>{
//             respData.message = "Error with blockchain response data".to_string();
//             respData.server_message =None;
//             respData.data = None;
//             ""
//             // return HttpResponse::BadRequest().json(respData);     
//         }
//     };

//     if(*code != "1"){
//         // blockchain request failed
//         respData.message = "".to_string()+mess;
//             respData.server_message =None;
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);     
//     }


//     // get user data from db after successful login
//     let user = UserService::get_by_(
//         &database.db,
//         doc! {"user_name":req_data.user_name.to_owned()}
//     ).await;

//     let user = match user {
//         Ok(user)=>{
//             match user {
//                 Some(data)=>{data},
//                 None=>{
//                     respData.message = "Could not find user".to_string();
//                     respData.server_message =None;
//                     respData.data = None;
//                     return HttpResponse::BadRequest().json(respData);
//                 }
//             }
//         },
//         Err(err)=>{
//             log::error!(" error getting user  {}", err.to_string());
//             respData.message = "Error getting user data".to_string();
//             respData.server_message =None;
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };


//     // generate token for user
//     let login_token =encode_token(
//         user.user_type, user.email,user.user_name);

//     let login_token = match login_token {
//         Ok(login_token)=>{login_token},
//         Err(err)=>{
//             log::error!(" error making login token {}", err.to_string());
//             return HttpResponse::InternalServerError().
//                 json(Response{message:"Error getting token".to_string()})
//         }
//     };


//     respData.message = "Ok".to_string();
//     respData.server_message =None;
//     respData.data = Some(login_token);
//     return HttpResponse::Ok().json(respData);


// }


// #[post("/user/kura_signup")]
// pub async fn kura_id_signup(database:Data<MongoService>, req_data:Json<CreateKuracoinID>)->HttpResponse {
//     let mut respData = GenericResp::<String>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: None
//     };
   
//     //validate request data
    
//     {
//         match req_data.borrow().validate() {
//             Ok(_) => {},
//             Err(err) => {
//                 log::error!(" validation error {}", err.to_string());
//                 respData.message = "Error validating request data".to_string();
//                 respData.server_message =Some(err.to_string());
//                 respData.data = None;
//                 return HttpResponse::BadRequest().json(respData);
//             }
//         }
//     }


//     // check if user exist
//     if check_if_user_exists_username(&database, &req_data.user_name.to_owned()).await{
//         log::info!(" User exists ");
//         respData.message = "User already exists".to_string();
//         respData.server_message =None;
//         respData.data = None;
//         return HttpResponse::BadRequest().json(respData);
//     }

    
 

//     // send message to the kuracoin blockchain to create new user
//     let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
//         Ok(data)=>{data.to_owned()},
//         Err(err)=>{
//             log::error!(" error getting vhenncoin server IP  {}", err.to_string());
//             respData.message = "Error connecting to blockchain".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };

//     let message_data = match serde_json::to_string(&req_data){
//         Ok(data)=>{data},
//         Err(err)=>{
//             log::error!(" error persing request data   {}", err.to_string());
//             respData.message = "Blockchain server error".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);  
//         }
//     };
//     let message = formatter::Formatter::request_formatter(
//         "CreateUserId".to_string(), 
//         message_data,
//         "".to_string(), 
//         "".to_string(),
//         "0".to_string());

//     let m = message.clone();
//     let ip = kura_coin_server_ip.clone();
//     let result = web::block(move || send_to_tcp_server(m,ip  )).await;
//     let response_string =match result {
//         Ok(data)=>{
//             match data {
//                 Ok(data)=>{data},
//                 Err(err)=>{
//                     log::info!(" error from tcp {}", err.to_string());
//                     respData.message = "Error from blockchain".to_string();
//                     respData.server_message =Some(err.to_string());
//                     respData.data = None;
//                     return HttpResponse::BadRequest().json(respData);     
//                 }
//             }
//         },
//         Err(err)=>{ 
//             log::error!(" error from blockchain tcp req {}", err.to_string());
//             respData.message = "Error persing data".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);   
//         }
//     };

//     let resp_data: Vec<&str>= response_string.split('\n').collect();
//     let code = match resp_data.get(0){
//         Some(data)=>{data},
//         None=>{
//             respData.message = "Error with blockchain response data".to_string();
//             respData.server_message =None;
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);     
//         }
//     };

//     if(*code != "1"){
//         // blockchain request failed
//         respData.message = "Failed to create digital ID on blockchain".to_string();
//             respData.server_message =None;
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);     
//     }




//     let user = User{
//         user_name:req_data.user_name.to_owned(),
//         created_at:chrono::offset::Utc::now().to_string(),
//         email:"".to_string(),
//         code:Option::from(93030),
//         user_type:UserType::User,
//         id:None
//     };

  
//     // setup
//     if user.user_type == UserType::User{
//         let user_res = UserService::create_user(database.db.borrow(),&user).await;

//         match user_res {
//             Ok(user)=> {},
//             Err(err)=>{
//                 log::error!(" error creating user {}", err.to_string());
//                 respData.message = "Error creating user".to_string();
//                 respData.server_message =None;
//                 respData.data = None;
//                 return HttpResponse::BadRequest().json(respData);  
//             }
//         }
//     }


//     // 

//     respData.message = "Ok".to_string();
//     respData.server_message =None;
//     respData.data = Some(req_data.user_name.to_owned());
//     return HttpResponse::Ok().json(respData);
// }

#[post("/friend_request/send")]
pub async fn send_friend_request(
    pool:Data<PgPool>,
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

    // get the user profile of the request user and check if the user has the new friend already
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
        status:"PENDING".to_string(),
        created_at:get_time_naive(),
        updated_at:get_time_naive(),
    };
    
    // make sure user does not send fr to himself
    if friend_request.user_name == claim.user_name.clone(){
        respData.message = "Cannot send friend request to self".to_string();
        respData.server_message = None;
        respData.data = None;
        return HttpResponse::BadRequest().json(respData); 
    }

    match FriendRequestService::create_friend_request(&pool, friend_request.clone()).await{
        Ok(data)=>{data}, 
        Err(ServiceError::FriendRequestExists)=>{
            respData.message = "Friend request already sent!".to_string();
            respData.server_message = None;
            respData.data = None;
            return HttpResponse::InternalServerError().json(respData);
        }
        Err(ServiceError::DatabaseError(err))=>{
            respData.message = "Error creating friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;
            log::error!("error creating friend request {}", err);
            return HttpResponse::InternalServerError().json(respData);
        }
        _ => {
            respData.message = "Error creating friend request".to_string();
            respData.server_message = None;
            respData.data = None;
            log::error!("unexpected error creating friend request ___",);
            return HttpResponse::InternalServerError().json(respData); 
        }
    };

    // get user profile
    let profile = match ProfileService::get_profile(&pool, req.user_name.clone()).await{
        Ok(profile)=>{profile},
        Err(err)=>{
            log::error!("error getting profile {}", err.to_string());
            Profile::default()
        }
    };

    if !profile.user_name.is_empty() && profile.app_f_token.is_some(){
        // send notification
        let payload = FcmMessage{
            message: MessagePayload {
                token: profile.app_f_token.unwrap_or_default() ,
                notification: Notification {
                    title: "New Friend Request".to_string(),
                    body: "@".to_owned() + &*friend_request.requester.clone()+ &*" just sent you a friend request. Go to your profile to view it!".to_string()
                },
                data: None,
            },
        };
        actix_web::rt::spawn(async move {
            send_app_notification(payload).await;  
        });
    }

    respData.message = "Ok".to_string();
    respData.server_message = None;
    respData.data = Some(friend_request);
    return HttpResponse::Ok().json(respData);

}


#[derive(Debug, Deserialize)]
struct GenID{id:String} 
#[get("/friend_request/accept/{id}")]
pub async fn accept_friend_request(
    pool:Data<PgPool>,
    path: web::Path<GenID>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<FriendRequest>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(FriendRequest::default())
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

  
 
    let fr = match FriendRequestService::accept_friend_request(&pool, path.id.clone(), claim.user_name.clone() ).await{
        Ok(data)=>{data}, 
        Err(err)=>{
            log::error!("error accepting request {}", err);
            respData.message = "Error accepting friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData);
        }
    };
    
   
    // get user profile
    let profile = match ProfileService::get_profile(&pool, fr.requester).await{
        Ok(profile)=>{profile},
        Err(err)=>{
            log::error!("error getting profile {}", err.to_string());
            Profile::default()
        }
    };

    if !profile.user_name.is_empty() && profile.app_f_token.is_some(){
        // send notification
        let payload = FcmMessage{
            message: MessagePayload {
                token: profile.app_f_token.unwrap_or_default() ,
                notification: Notification {
                    title: "New Friend Request".to_string(),
                    body: "@".to_owned() + &*fr.user_name.clone()+ &*" Has accepted your friend request!".to_string()
                },
                data: None,
            },
        };
        actix_web::rt::spawn(async move {
            send_app_notification(payload).await;
        });
    }

    respData.message = "Ok".to_string();
    respData.server_message =None;
    respData.data = None;
    return HttpResponse::Ok().json(respData);

}



#[get("/friend_request/reject/{id}")]
pub async fn reject_friend_request(
    pool:Data<PgPool>,
    path: web::Path<GenID>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<FriendRequest>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(FriendRequest::default())
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
    
    match FriendRequestService::reject_friend_request2(&pool, path.id.clone(), claim.user_name.clone()).await{
        Ok(data)=>{data}, 
        Err(err)=>{
            log::error!("error rejecting FR {}", err);
            respData.message = "Error rejecting friend request".to_string();
            respData.server_message = Some(err.to_string());
            respData.data = None;

            return HttpResponse::InternalServerError().json(respData);
        }
    };

    respData.message = "Ok".to_string();
    respData.server_message =None;
    respData.data = None;
    return HttpResponse::Ok().json(respData);

}



#[get("/friend_requests")]
pub async fn get_my_friend_request(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse{
    let mut respData = GenericResp::<Vec<FriendRequestWithProfile>>{
        message:"".to_string(),
        server_message: Some("".to_string()),
        data: Some(vec![FriendRequestWithProfile::default()])
    };

    let claim = match claim {
        Some(claim)=>{claim},
        None=>{
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized()
                .json(respData)
        }
    };

    // get friend request 
    let fr = match FriendRequestService::get_user_friend_request(&pool, claim.user_name.clone()).await{
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

// delete my account
#[get("/delete")]
pub async fn delete_profile(
    pool:Data<PgPool>,
    claim:Option<ReqData<Claims>>
)->HttpResponse {
    let mut respData = GenericResp::<String> {
        message: "".to_string(),
        server_message:None,
        data: None
    };

    //     //claim
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
    
    match UserService::delete_user(&pool, claim.user_name.clone()).await{
        Ok(data)=>{
            respData.message = "Ok".to_string();
            return HttpResponse::Ok().json(respData);
        },
        Err(err)=>{
            log::error!("error deleting user {}", err);
            respData.message = "Error deleting user".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(respData);
        }
    }
}
