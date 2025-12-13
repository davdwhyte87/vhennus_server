
use thiserror::Error;
#[derive(Error, Debug)]
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