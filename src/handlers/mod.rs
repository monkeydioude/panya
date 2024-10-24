use std::error::Error as stdError;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

pub mod channel;
pub mod feed;
pub mod healthcheck;
pub mod json;
pub mod panya;

#[derive(Debug)]
pub enum ApiTokenError {
    Missing,
    Invalid,
}

pub struct Token(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ApiTokenError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            Some(token) => Outcome::Success(Token(token.to_string())),
            None => Outcome::Error((Status::Unauthorized, ApiTokenError::Invalid)),
        }
    }
}

pub struct XQueryID(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for XQueryID {
    type Error = Box<dyn stdError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cache = req.local_cache(|| ("".to_string(), 0 as u128));
        Outcome::Success(XQueryID(cache.0.clone()))
    }
}
