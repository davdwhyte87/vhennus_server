
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("error with siging transaction")]
    SignTransactionError,

    #[error("Error with data")]
    SerializationError,

    #[error("Error error sending blockchain request")]
    BlockChainRequestError,
}