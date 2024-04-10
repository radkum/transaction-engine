use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

use crate::{
    engine::{transaction::TransactionInfo, ClientId},
    EngineError::SendTransactionError,
};

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("ClientNotExists: {0}")]
    ClientNotExists(ClientId),
    #[error("CsvError: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Failed to get wallet from client. THIS SHOULD NOT HAPPEN")]
    FailedToGetWallet,
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("InputFileError: {0}")]
    InputFileError(String),
    #[error("JoinError: {0}")]
    JoinError(#[from] JoinError),
    #[error("RecordError: {0}")]
    RecordError(String),
    #[error("SendTransactionError: {0}")]
    SendTransactionError(String),
}

impl From<SendError<TransactionInfo>> for EngineError {
    fn from(value: SendError<TransactionInfo>) -> Self {
        SendTransactionError(format!("{:?}", value.0))
    }
}
