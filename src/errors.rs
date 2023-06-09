use std::{error::Error, io};

pub type Result<T> = std::result::Result<T, ApplicationError>;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("IO Error: {0:?}")]
    IoError(#[from] io::Error),
    #[error("Generic error: {0:?}")]
    GeneralError(#[from] Box<dyn Error>),
    #[error("PathError")]
    PathError,
    #[error("NotFound")]
    NotFound,
}
