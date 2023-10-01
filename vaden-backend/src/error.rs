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

    #[error("unable to hash {0}")]
    UnableToHashPassword(#[from] scrypt::password_hash::Error),

    #[error("unable to read YAML {0}")]
    UnableToReadYaml(#[from] yaml_rust::ScanError),
}
