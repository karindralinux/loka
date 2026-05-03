use thiserror::Error;

pub type Result<T> = std::result::Result<T, LokaError>;

#[derive(Debug, Error)]
pub enum LokaError {
    #[error("not supported: {0}")]
    NotSupported(String),

    #[error("connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("internal error: {0}")]
    Internal(String),
}