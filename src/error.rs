use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Bad data: {0})")]
    BadData(String),
    #[error("Internal")]
    Internal(String),
    #[error("IoError: {0}")]
    IoError(String),
    #[error("Reqwest: {0}")]
    Reqwest(String),
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::BadData(err.to_string())
    }
}
