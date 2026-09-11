use std::env;

use dotenv::dotenv;
use log::error;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: String,
    pub database_url: String,
    pub email:String,
    pub email_password:String,
    pub exchange_rate_api_key:String,
    pub blockchain_ip:String,
    pub earnings_wallet:String,
    pub earnings_wallet_password:String,
    pub app_env:String,
    pub blockchain_address:String,
    pub send_plus_client_id:String,
    pub send_plus_client_secrete:String
}


pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let current_dir = std::env::current_dir().unwrap();
    error!("Current directory: {:?}", current_dir);

    // Log environment variables
    dotenv().ok();
    error!("PORT: {:?}", env::var("PORT"));
    error!("App Env: {:?}", env::var("APP_ENV"));
    let port = match env::var("PORT"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env port {}", err.to_string());
            "8000".to_string();
            panic!()
        }
    };

    let database_url = match env::var("DATABASE_URL"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env database url {}", err.to_string());
            panic!()
        }
    };

    let email = match env::var("EMAIL"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env email {}", err.to_string());
            panic!()
        }
    };
    let email_password = match env::var("EMAIL_PASSWORD"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("error loading env email password {}", err.to_string());
            panic!()
        }
    };
    let exchange_rate_api_key = match env::var("EXCHANGE_API_KEY"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("env error loading exchange api key {}", err.to_string());
            panic!()
        }
    };    
    let earnings_wallet_password = match env::var("EARNINGS_WALLET_PASSWORD"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("env error loading error wallet password {}", err.to_string());
            panic!()
        }
    };
    let earnings_wallet = match env::var("EARNINGS_WALLET"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("env error loading earnings wallet {}", err.to_string());
            panic!()
        }
    };
    let blockchain_ip = match env::var("BLOCKCHAIN_IP"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("env error loading blockchain ip {}", err.to_string());
            panic!()
        }
    };
    let app_env = match env::var("APP_ENV"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("env error loading app env{}", err.to_string());
            panic!()
        }
    };
    let blockchain_address = match env::var("BLOCKCHAIN_ADDRESS"){
        Ok(data)=>{
            data
        },
        Err(err)=>{
            error!("env error loading app env blockchain address {}", err.to_string());
            panic!()
        }
    };
    let send_plus_client_id = match env::var("SENDPULSE_CLIENT_ID"){
        Ok(data)=>{data},
        Err(err)=>{
            error!("env error loading send plus-client id {}", err.to_string());
            panic!()
        }
    };
    let send_plus_client_secrete = match env::var("SENDPULSE_CLIENT_SECRET"){
        Ok(data)=>{data},
        Err(err)=>{
            error!("env error loading send plus-client secrete {}", err.to_string());
            panic!()
        }
    };
    Config{
        port: port,
        email:email,
        database_url:database_url,
        email_password:email_password,
        exchange_rate_api_key: exchange_rate_api_key,
        earnings_wallet,
        earnings_wallet_password,
        blockchain_ip,
        app_env, 
        blockchain_address,
        send_plus_client_id,
        send_plus_client_secrete
    }
});