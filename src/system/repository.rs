use std::error::Error;

use sqlx::PgPool;

use crate::system::models::System;

pub const SYSTEM_COLLECTION: &str = "System";

pub struct SystemRepo;

impl SystemRepo {
    pub async fn get_system_data(pool: &PgPool) -> Result<Option<System>, Box<dyn Error>> {
        let data = sqlx::query_as!(System,
            "SELECT * FROM system_data WHERE id=$1 ", 1)
            .fetch_optional(pool).await?;
        Ok(data)
    }

    pub async fn update_system_data(pool: &PgPool, system: System) -> Result<(), Box<dyn Error>> {
        let res = sqlx::query_as!(System,
            "UPDATE system_data 
            SET 
                ngn = COALESCE($2, ngn),
                price = COALESCE($3, price)
            WHERE id=$1
             ", 1, system.ngn, system.price).execute(pool).await?;

        Ok(())
    }
}