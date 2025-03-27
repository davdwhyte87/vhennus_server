use std::error::Error;
use chrono::Utc;
use jsonwebtoken::{encode, decode, EncodingKey, Header, DecodingKey, Validation};
use serde_derive::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims{
    pub role:String,
    pub email: String,
    pub user_name:String,
    pub exp:usize
}



pub fn encode_token(role: String, email:String, name:String) ->Result<String, Box<dyn Error>>{
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .expect("valid timestamp")
        .timestamp();
    let my_claims =Claims{
        role:role,
        email:email,
        user_name:name,
        exp:expiration as usize
    };
    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret("secret".as_ref()))?;
    return Ok(token)

}

pub fn decode_token(token_string:String)->Result<Claims, Box<dyn Error>>{
    let token = decode::<Claims>(
        &token_string,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default())?;
    Ok(token.claims)
}