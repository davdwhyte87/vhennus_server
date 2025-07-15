use sqlx::PgPool;
use crate::lottery::models::{LotteryTransaction};

pub struct LotteryRepo{

}

impl LotteryRepo{
    pub async fn create_transaction(pool: &PgPool, trans:LotteryTransaction){
        let res = sqlx::query!(LotteryTransaction,
            "
            INSERT INTO lottery_transaction VALUES ()
            "
        )
    }
}