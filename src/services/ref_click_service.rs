use std::error::Error;
use sqlx::PgPool;
use crate::models::ref_click::RefClick;

pub struct RefClickService {
    
}

impl RefClickService {
    pub async fn create_ref_click(pool:&PgPool, click_id:String, code:String)->Result<RefClick, Box<dyn Error>>{
        let res = sqlx::query_as!(RefClick,
            "INSERT INTO ref_clicks (click_id, code) 
             VALUES ($1,$2)
             RETURNING click_id,code, created_at
             ",
             click_id, code
        ).fetch_one(pool).await?;
        Ok(res)
    }

    pub async fn get_ref_click(pool:&PgPool, click_id:String)->Result<RefClick, Box<dyn Error>>{
        let res = sqlx::query_as!(RefClick,
            "SELECT * FROM ref_clicks 
             WHERE click_id = $1",
             click_id
        ).fetch_one(pool).await?;
        Ok(res)
    }
}