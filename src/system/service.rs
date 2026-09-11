use std::error::Error;

use sqlx::PgPool;

use crate::system::models::System;
use crate::system::repository::SystemRepo;

pub struct SystemService;

impl SystemService {
    pub async fn get_system_data(pool: &PgPool) -> Result<Option<System>, Box<dyn Error>> {
        SystemRepo::get_system_data(pool).await
    }

    pub async fn update_system_data(pool: &PgPool, system: System) -> Result<(), Box<dyn Error>> {
        SystemRepo::update_system_data(pool, system).await
    }
}