use std::borrow::{Borrow, BorrowMut};
use actix_web::{Responder, get, HttpResponse, web::Json, post};

use actix_web::web::{Data, ReqData};
use handlebars::Handlebars;
use rand::Rng;
use validator::Validate;
use crate::database::db::db::DB;
use crate::models::helper::EmailData;
use crate::models::power_up::{PlayerPowerUp, PowerUpType};
use crate::models::request_models::{GetCodeReq, LoginReq};
use crate::models::response::{CodeResp, LoginResp, PlayerRunInfoRes, Response};
use crate::models::run_info::RunInfo;
use crate::models::user::{User, UserType};
use crate::models::wallet::Wallet;
use crate::req_models::create_user_req::CreateUserReq;
use crate::services::mongo_service::MongoService;
use crate::services::power_up_service::PowerUpService;
use crate::services::run_info_service::RunInfoService;
use crate::services::user_service::UserService;
use crate::services::wallet_service::WalletService;
use crate::utils::auth::{Claims, decode_token, encode_token};
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


// seed run info data for player in the database
pub async fn create_player_stats(database:&Data<MongoService>, email:&String)->bool{
    let run_info = RunInfo{
        id:None,
        created_at:chrono::offset::Utc::now().to_string(),
        updated_at: chrono::offset::Utc::now().to_string(),
        distance: 0,
        high_score:0,
        user_email:email.to_string()
    };

    let create_run_info_res = RunInfoService::create(database.db.borrow(), &run_info).await;
    match create_run_info_res {
        Ok(_)=>{},
        Err(_)=>{return false}
    }
    return true

}


// seed wallet data
pub async fn create_player_wallet(database:&Data<MongoService>, email:&String)->bool{
    let mut ok = false;
    let wallet = Wallet{
        user_email:email.to_string(),
        amount:0,
        created_at:chrono::offset::Utc::now().to_string(),
        id:None
    };
    let player_wallet = WalletService::create(&database.db,&wallet).await;
    match player_wallet {
        Ok(_)=>{
            ok=true;
        },
        Err(err)=>{
            ok=false;
        }
    }
    return ok;
}

// seed power up info for player in db
pub async fn create_player_powerups (database:&Data<MongoService>, email:&String)->bool{
    let mut ok = false;
    // phasing powerup
    let user_power_up = PlayerPowerUp{
        id:None,
        created_at:chrono::offset::Utc::now().to_string(),
        amount:0,
        in_game_amount:0,
        power_up_type: PowerUpType::Phasing,
        user_email:email.to_string()
    };
    let create_res = PowerUpService::create_player_powerup(&database.db, &user_power_up).await;
    let create_res = match create_res {
        Ok(_)=>{ok = true},
        Err(_)=>{
           ok = false;
        }
    };

    // blast power up
    let user_power_up = PlayerPowerUp{
        id:None,
        created_at:chrono::offset::Utc::now().to_string(),
        amount:0,
        in_game_amount:0,
        power_up_type: PowerUpType::Blast,
        user_email:email.to_string()
    };
    let create_res = PowerUpService::create_player_powerup(&database.db, &user_power_up).await;
    let create_res = match create_res {
        Ok(_)=>{ok = true},
        Err(_)=>{
            ok = false;
        }
    };

    // slow motion power up

    let user_power_up = PlayerPowerUp{
        id:None,
        created_at:chrono::offset::Utc::now().to_string(),
        amount:0,
        in_game_amount:0,
        power_up_type: PowerUpType::SlowMotion,
        user_email:email.to_string()
    };
    let create_res = PowerUpService::create_player_powerup(&database.db, &user_power_up).await;
    let create_res = match create_res {
        Ok(_)=>{ok = true},
        Err(_)=>{
            ok = false;
        }
    };

    return ok;
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