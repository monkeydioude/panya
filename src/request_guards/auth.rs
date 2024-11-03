use crate::config::Settings;
use crate::entities::user::User;
use crate::services::grpc::jwt_status;
use crate::services::token::ApiTokenError;
use crate::utils::decode_base64_url;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use serde::Deserialize;

use super::xqueryid::XQueryID;

#[derive(Debug)]
pub struct Auth {
    pub user: User,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Claims {
    pub uid: i32,
    pub expire: i128,
    pub refresh: i128,
}

impl From<Claims> for Auth {
    fn from(claims: Claims) -> Self {
        Auth {
            user: User {
                id: claims.uid,
                channel_ids: vec![],
            },
        }
    }
}

pub fn fetch_claims(token: &str) -> Option<Claims> {
    let parts: Vec<&str> = token.split(".").collect();
    if parts.len() != 3 {
        return None;
    }
    decode_base64_url(parts[1]).ok()
}

// this request guard does, in this order and if stop when an error occurs in one of those steps:
// - fetch the JWT from the header
// - validate the JWT by calling the identity service
// - parse the payload for the user's information
// - returns an Auth struct containing an User entity
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ApiTokenError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match req.headers().get_one("Authorization") {
            Some(_token) => _token,
            None => return Outcome::Error((Status::Unauthorized, ApiTokenError::Invalid)),
        };

        let settings = match req.guard::<&State<Settings>>().await {
            rocket::outcome::Outcome::Success(_settings) => _settings,
            _ => return Outcome::Error((Status::InternalServerError, ApiTokenError::OtherError)),
        };
        let uuid = match req.guard::<XQueryID>().await {
            rocket::outcome::Outcome::Success(_uuid) => _uuid,
            _ => return Outcome::Error((Status::InternalServerError, ApiTokenError::OtherError)),
        };
        if let Err(err) = jwt_status(&settings.identity_server_addr, token).await {
            eprintln!("({}) {}", uuid, err);
            return Outcome::Error((Status::Unauthorized, ApiTokenError::Invalid));
        }

        let claims = match fetch_claims(token) {
            Some(_claims) => _claims,
            None => return Outcome::Error((Status::Unauthorized, ApiTokenError::Invalid)),
        };

        rocket::outcome::Outcome::Success(claims.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::request_guards::auth::fetch_claims;

    #[test]
    fn test_i_can_fetch_claims() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHBpcmUiOjE3MzA0ODYwNTEsInJlZnJlc2giOjE3MzEwODcyNTEsInVpZCI6N30.uTK-qeOM6yEdjncLACGAth75GC6GFJUsVNzNFIO1IcU";
        assert_eq!(7, fetch_claims(token).unwrap().uid);
    }
}
