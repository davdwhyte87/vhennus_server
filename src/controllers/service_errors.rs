use sqlx::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("User does not exist")]
    UserNotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Friend request already exists")]
    FriendRequestExists,
    
    #[error("Could not update data")]
    NoUpdatedRow,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}