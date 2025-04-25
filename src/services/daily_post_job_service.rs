use std::fmt::Debug;
use std::io::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use actix_rt::time::sleep;
use chrono::{DateTime, Utc};
use cron::Schedule;
use log::log;

use rand::thread_rng;
use sqlx::{PgPool, Pool, Postgres};
use crate::services::app_notify::{send_app_notification, FcmMessage, MessagePayload, Notification};
use crate::services::profile_service::ProfileService;
use crate::services::user_service::UserService;
use rand::seq::SliceRandom;
use rand::prelude::*;
use crate::services::post_service::PostService;

pub async fn daily_post_cron_task() {
    let expression = "0 0 8,20 * * 1,3"; // Runs at the start of every hour

    let schedule = Schedule::from_str(expression).unwrap();

    loop {
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).next() {
            let duration = (next - now).to_std().unwrap_or(Duration::from_secs(0));
            sleep(duration).await;
            log::info!("Running cron job at {:?}", Utc::now());

            // Perform the task here (e.g., database cleanup, API request, etc.)
            // get all users and get
            // send message "checj out wha t your friends are posting"

            let messages = vec!["Hey there! Checkout what your friends are talking about.",
                                "There's so much happening on vhennus right now, check it out",
                                "You need to connect... Try meeting like minded people on vhennus"
            ];
        }
    }
}

pub async fn morning_notify(pool:PgPool)->Result<(), Box<dyn std::error::Error>>{
    let cron_expression = "15 11 * * * *"; // Cron expression for 11:10 AM
    let schedule = match Schedule::from_str(cron_expression) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing cron expression: {}", e);
            return Err(Box::new(e));
        }
    };

    loop {
        let now: DateTime<Utc> = Utc::now();
        let next_occurrence: DateTime<Utc> = match schedule.upcoming(Utc).next() {
            Some(n) => n,
            None => {
                eprintln!("Error: Could not retrieve next scheduled time.");
                continue;
            }
        };
        
        use chrono::{DateTime, Duration, Utc};
        if next_occurrence > now {
            let sleep_duration: Duration = next_occurrence - now;
            let sleep_std_duration = match sleep_duration.to_std() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error converting duration: {}", e);
                    continue;
                }
            };

            println!("Sleeping until next scheduled run at: {}", next_occurrence);
            thread::sleep(sleep_std_duration);
            println!("Running scheduled task at: {}", Utc::now());

            let profiles =match  ProfileService::get_all(&pool).await{
                Ok(p) => p,
                Err(err)=>{
                    log::error!("error getting all users  {}", err.to_string());
                    continue;
                }
            };


            #[derive(Debug)]
            struct Msg{
                title: String,
                description: String,
            }
            let messages:Vec<Msg> = vec![
                Msg{title: String::from("Everyone is a creator"),description:String::from("Make 500 VEC everytime you post!")},
                Msg{title: String::from("Earn over 5,000 naira daily"),description:String::from("Make 10 VEC every minute you spend on the app!")},
                Msg{title: String::from("Lets stack those coins"),description:String::from("Make 500 VEC everytime you post!")}
            ];
            let mut rng = thread_rng();
            let message =match messages.choose(&mut rng) {
                Some(message)=>{message},
                None=>{
                    continue;
                }
            };

            for profile in profiles {
                if profile.app_f_token.is_some(){
                    // send
                    let payload = FcmMessage{
                        message: MessagePayload {
                            token: profile.app_f_token.unwrap_or_default() ,
                            notification: Notification {
                                title: message.title.clone(),
                                body: message.description.clone()
                            },
                            data: None,
                        },
                    };
                    send_app_notification(payload).await;
                }
            }
         
        } else {
            eprintln!("Error: Next scheduled time is in the past. This should not happen.");
            continue;
        }
    }
}

pub async fn daily_coin_post_cron_task(db_pool: PgPool)->Result<(), Box<dyn std::error::Error>> {
    //let expression = " 0 0 8,20 * * 0,2,4,5,6"; // Runs at the start of every hour
    let expression = "0   50   9     *       *  *  *";
    let schedule = match Schedule::from_str(expression){
        Ok(schedule) => schedule,
        Err(e)=>{
            log::error!("scheduler error {}", e);
            return Err(Box::new(e));
        }
    };

    loop {
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).next() {
            let duration = (next - now).to_std().unwrap_or(Duration::from_secs(0));
            sleep(duration).await;
            log::info!("Running cron job at {:?}", Utc::now());

            // Perform the task here (e.g., database cleanup, API request, etc.)
            log::error!("starting daily coin task");
            // get all users and get
            // send message "checj out wha t your friends are posting"
           let profiles =match  ProfileService::get_all(&db_pool).await{
               Ok(p) => p,
               Err(err)=>{
                   log::error!("error getting all users  {}", err.to_string());
                   continue;
               }
           };


          
            let messages:Vec<Msg> = vec![
                Msg{title: String::from("Everyone is a creator"),description:String::from("Make 500 naira everytime you post!")},
                Msg{title: String::from("Earn over 5,000 naira daily"),description:String::from("Make 500 naira everytime you post!")},
                Msg{title: String::from("Lets stack those coins"),description:String::from("Make 500 naira everytime you post!")}
            ];
            let mut rng = thread_rng();
            let message =match messages.choose(&mut rng) {
                Some(message)=>{message},
                None=>{
                    continue;
                }
            };
          
            for profile in profiles {
                if profile.app_f_token.is_some(){
                    // send
                    let payload = FcmMessage{
                        message: MessagePayload {
                            token: profile.app_f_token.unwrap_or_default() ,
                            notification: Notification {
                                title: message.title.clone(),
                                body: message.description.clone()
                            },
                            data: None,
                        },
                    };
                    send_app_notification(payload).await;
                }
            }
            
        }
    }
}


pub async fn onehr_comment_cron_taskx(pool: PgPool)->Result<(), Box<dyn std::error::Error>> {
    loop {
        log::error!("starting daily coin task");
        sleep(Duration::from_secs(180)).await;
        // Perform the task here (e.g., database cleanup, API request, etc.)
        // get all comments and post owners to notify
        
         let profiles = match PostService::get_last_1hr_comments(&pool).await{
             Ok(p) => p,
             Err(err)=>{
                 log::error!("Error getting comments and post owners from past hour for notify {}", err);
                 continue;
             }
         };
        let message = Msg{title:String::from("Your post is getting attention 🎉"),
            description:String::from("You've received new comments on your post. Join the conversation!"),
        };
        for profile in profiles {
            
            if profile.token.is_some(){
                // send
                let payload = FcmMessage{
                    message: MessagePayload {
                        token: profile.token.unwrap_or_default() ,
                        notification: Notification {
                            title: message.title.clone(),
                            body: message.description.clone()
                        },
                        data: None,
                    },
                };
                send_app_notification(payload).await;
            }
        }
    }
}

pub async fn start_jobs(db_pool: PgPool){
    log::info!("✅ Scheduling cron job...");
    //let pool = db_pool.clone();
    //actix_rt::spawn(daily_post_cron_task());
    actix_rt::spawn(onehr_comment_cron_taskx(db_pool.clone()));
    actix_rt::spawn(morning_notify(db_pool.clone()));
    log::info!("✅ Cron job has been spawned.");
}

#[derive(Debug)]
struct Msg{
    title: String,
    description: String,
}