use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error, Responder, Clone)]
pub struct Error(pub String);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Error(value.to_string())
    }
}

impl Error {
    pub fn str(str: &str) -> Self {
        Error(str.to_string())
    }

    pub fn str_to_result<T>(str: &str) -> Result<T, Error> {
        Err(Self::str(str))
    }

    pub fn to_result<T>(&self) -> Result<T, Error> {
        Err(self.clone())
    }
}
