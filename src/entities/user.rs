use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use serde::{Deserialize, Serialize};

use crate::db::model::{CloneEntity, CollectionModel, Updatable};
use crate::db::mongo::Handle;
use crate::db::user::Users;
use crate::request_guards::auth::Auth;
use crate::request_guards::xqueryid::XQueryID;
use crate::{
    db::model::{FieldSort, PrimaryID},
    error::Error,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(default)]
    pub channel_ids: Vec<i32>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = match req.guard::<Auth>().await {
            Outcome::Success(_auth) => _auth,
            Outcome::Error(err) => return Outcome::Error((err.0, err.1.into())),
            _ => {
                return Outcome::Error((
                    Status::InternalServerError,
                    Error("unhandled behavior with auth guard".to_string()),
                ))
            }
        };
        let handle = match req.guard::<&State<Handle>>().await {
            Outcome::Success(_settings) => _settings,
            _ => {
                return Outcome::Error((
                    Status::InternalServerError,
                    Error("no db handle".to_string()),
                ))
            }
        };
        let uuid = match req.guard::<XQueryID>().await {
            Outcome::Success(_uuid) => _uuid,
            _ => XQueryID("no x-query-token".to_string()),
        };
        let users_coll = match Users::<User>::new(handle, "panya") {
            Ok(_coll) => _coll,
            Err(err) => {
                return Outcome::Error((Status::InternalServerError, Error(err.to_string())))
            }
        };
        match users_coll.find_by_field("id", auth.user_id).await {
            Some(_user) => Outcome::Success(_user),
            None => {
                eprintln!("({}) could not find any user in db", uuid);
                Outcome::Error((Status::BadRequest, Error::str("user does not exist")))
            }
        }
    }
}

impl FieldSort<String> for User {
    fn sort_by_value(&self) -> String {
        self.id.to_string()
    }
}

impl PrimaryID<i32> for User {
    fn get_primary_id(&self) -> Option<i32> {
        Some(self.id)
    }
}

impl Updatable<i32, User> for User {
    fn update(&self, _: User) -> User {
        (*self).clone()
    }
}

impl CloneEntity<User> for User {
    fn clone_entity(&self) -> User {
        (*self).clone()
    }
}
