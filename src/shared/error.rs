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

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found {0} - {1}")]
    NotFoundError(String, String),
    #[error("error with siging transaction")]
    SignTransactionError,

    #[error("Error with data")]
    SerializationError,

    #[error("Error error sending blockchain request")]
    BlockChainRequestError,
    
    //database errorrs ---- 
    #[error("Error with database")]
    CreateTransactionError,
    #[error("Error inserting data into db")]
    DBInsertError,
    #[error("Error updating data in db")]
    DBUpdateError,
    #[error("Error deleting data from db")]
    DBDeleteError,
    #[error("Error getting data from db")]
    FetchDataError,
    #[error("Data already exists in database")]
    AlreadyExistsError,
    
    
    // http errors
    #[error("Unauthorized action")]
    UnauthorizedError,
    #[error("Request data error")]
    RequestDataError,
    #[error("Bad request {0}")]
    BadRequestError(String),

    // email errors
    #[error("Error sending email")]
    SendMailError
}