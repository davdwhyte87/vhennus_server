use std::error::Error;
use mongodb::results::{InsertOneResult, UpdateResult};
use mongodb::Database;
use crate::wallet::models::Wallet;
use crate::wallet::repository::WalletRepo;

pub struct WalletService;

impl WalletService {
    pub async fn create(db: &Database, data: &Wallet) -> Result<InsertOneResult, Box<dyn Error>> {
        WalletRepo::create(db, data).await
    }

    pub async fn get_by_email(db: &Database, email: &String) -> Result<Option<Wallet>, Box<dyn Error>> {
        WalletRepo::get_by_email(db, email).await
    }

    pub async fn update(db: &Database, id: String, new_data: &Wallet) -> Result<UpdateResult, Box<dyn Error>> {
        WalletRepo::update(db, id, new_data).await
    }
}
