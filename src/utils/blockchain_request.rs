use actix_web::HttpResponse;
use awc::Client;
use log::{debug, error};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::CONFIG;
use crate::models::app_error::AppError;
use crate::models::response::BResponse;

pub async fn send_to_blockchain <T:DeserializeOwned, K:Serialize>(req_data:K, url:String) ->Result<BResponse<T>, AppError>{
    // send request
    let serialized_payload =match serde_json::to_string(&req_data) {
        Ok(serialized_payload)=>{serialized_payload},
        Err(err)=>{
            error!("{}", err);
            return Err(AppError::SerializationError)
        }
    };
    let client = Client::new();
    let url = format!("{}{}", CONFIG.blockchain_address, url);
    debug!("url: {}", url);
    debug!("payload: {}", serialized_payload);
    let response = client
        .post(url.as_str())
        .insert_header(("Content-Type", "application/json")) // Required for FCM
        .send_body(serialized_payload) // Send the serialized struct as JSON
        .await;
    let res_data= match response {
        Ok(mut data)=>{
            let bytes = match data.body().await {
                Ok(b) => b,
                Err(err) => {
                    error!("Failed to read response body: {}", err);
                    return Err(  return Err(AppError::SerializationError))
                }
            };

            // Convert bytes to string
            let raw_text = String::from_utf8_lossy(&bytes).to_string();
            debug!("{}", raw_text);
            let res:BResponse<T> = match serde_json::from_str(&raw_text) {
                Ok(d)=>{d},
                Err(err)=>{
                    error!("{}", err);
                    return Err(  return Err(AppError::SerializationError))
                }
            };
            res
        },
        Err(err)=>{
            error!("blockchian message error {}", err);
            return Err(  return Err(AppError::BlockChainRequestError))
        }
    };
    
    Ok( res_data )
}