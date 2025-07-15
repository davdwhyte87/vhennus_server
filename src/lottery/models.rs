use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct LotteryTransactions {
    pub id: String,
    pub user_name:String,
    pub status:LotteryTransactionStatus,
    pub amount: i64,
    pub number_of_tickets:i64,
    pub transaction_id:String,
    pub created_at:NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct LotteryTickets {
    pub user_name:String,
    pub number:i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct LotteryGames{
    pub id: String,
    pub status:LotteryGameStatus,
    pub amount: i64,
    pub winning_number:Option<i64>,
    pub created_at:NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Type, Default)]
#[sqlx(type_name = "lottery_game_status", rename_all = "lowercase")]
pub enum LotteryGameStatus {
    #[default]
    Ongoing,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Type, Default)]
#[sqlx(type_name = "lottery_transaction_status", rename_all = "lowercase")]
pub enum LotteryTransactionStatus {
    #[default]
    Pending,
    Done,
    Failed,
}


