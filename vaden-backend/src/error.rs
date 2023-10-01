use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, VadenError>;

#[derive(Debug, Error)]
pub enum VadenError {
    #[error("invalid config {0}")]
    InvalidConfig(String),

    #[error("unable to read {0}")]
    UnableToRead(#[from] io::Error),

    #[error("unable to format {0}")]
    UnableToFormat(#[from] std::fmt::Error),
}
