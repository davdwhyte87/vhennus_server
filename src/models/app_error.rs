
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("error with siging transaction")]
    SignTransactionError,
}