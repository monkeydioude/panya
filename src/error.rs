use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Request, Response,
};
use std::{error::Error as stdErr, fmt::Display, io::Cursor};
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

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
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

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error(value)
    }
}

impl From<Box<dyn stdErr>> for Error {
    fn from(value: Box<dyn stdErr>) -> Self {
        Error(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error(value.to_string())
    }
}

pub enum HTTPError {
    BadRequest(Error),
    Unauthorized(Error),
    InternalServerError(Error),
}

impl HTTPError {
    fn get_http_status(&self) -> Status {
        match self {
            HTTPError::BadRequest(_) => Status::BadRequest,
            HTTPError::Unauthorized(_) => Status::Unauthorized,
            HTTPError::InternalServerError(_) => Status::InternalServerError,
            // _ => Status::BadRequest,
        }
    }

    fn get_value(&self) -> Error {
        match self {
            HTTPError::BadRequest(v) => v.clone(),
            HTTPError::Unauthorized(v) => v.clone(),
            HTTPError::InternalServerError(v) => v.clone(),
        }
    }
}

impl From<Error> for HTTPError {
    fn from(value: Error) -> Self {
        error!("{}", value);
        Self::InternalServerError(value)
    }
}

impl From<mongodb::error::Error> for HTTPError {
    fn from(value: mongodb::error::Error) -> Self {
        error!("{}", value);
        Self::InternalServerError(Error(value.to_string()))
    }
}

impl<'r> Responder<'r, 'static> for HTTPError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(self.get_http_status())
            .header(ContentType::JSON)
            .sized_body(
                self.get_value().to_string().len(),
                Cursor::new(self.get_value().to_string()),
            )
            .ok()
    }
}
