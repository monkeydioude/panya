use rocket::serde::json::Json;

use crate::{entities::user::User, error::HTTPError, request_guards::xqueryid::XQueryID};

// #[derive(Deserialize, Serialize)]
// pub struct AddUser {
//     username: String,
//     #[serde(skip_deserializing)]
//     email: String,
// }

// // /panya/channel
// #[post("/user", format = "json", data = "<add_user>")]
// pub async fn add_user(
//     handle: &rocket::State<Handle>,
//     add_user: Json<AddUser>,
//     settings: &rocket::State<Settings>,
//     uuid: XQueryID,
// ) -> Result<Json<HTTPResponse>, HTTPError> {
//     Ok(Json(HTTPResponse::created()))
// }

// /panya/channel
#[get("/user/me")]
pub async fn show_user_me(_uuid: XQueryID, user: User) -> Result<Json<User>, HTTPError> {
    Ok(Json(user))
}
