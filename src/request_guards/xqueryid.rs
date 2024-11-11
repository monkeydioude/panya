use std::error::Error as stdError;
use std::fmt::Display;

use rocket::request::{FromRequest, Outcome};
use rocket::Request;

#[derive(Debug)]
pub struct XQueryID(pub String);

impl Display for XQueryID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// this request guard inject the x query id from fairings
// into handlers parameters
#[rocket::async_trait]
impl<'r> FromRequest<'r> for XQueryID {
    type Error = Box<dyn stdError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cache = req.local_cache(|| ("".to_string(), 0 as u128));
        Outcome::Success(XQueryID(cache.0.clone()))
    }
}
