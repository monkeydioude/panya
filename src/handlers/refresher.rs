use rocket::serde::json::Json;
use serde::Deserialize;
use crate::{config::Settings, db::{entities::{AscDesc, Refresh}, model::{BlankCollection, CollectionModel}, mongo::Handle}, error::Error as localError};
use std::error::Error;

#[derive(Deserialize)]
pub struct Refresher {
    pub refresh_frequency: Option<AscDesc>,
    pub fetch_avg: Option<AscDesc>,
    pub no_fetch_avg: Option<bool>,
    pub limit: Option<i32>,
}

fn handle_error(err: &dyn Error, msg: &str)-> Result<String, rocket::http::Status> {
    error!(
        "{}: {}",
        msg,
        err
    );
    Err(rocket::http::Status::InternalServerError)
}

#[post("/refresher", data="<payload>")]
pub async fn post_refresher(
    handle: &rocket::State<Handle>,
    settings: &rocket::State<Settings>,
    payload: Json<Refresher>,
) -> Result<String, rocket::http::Status> {
    let refresh_coll = match BlankCollection::<Refresh>::new(handle, "panya", "refresh") {
        Ok(c) => c,
        Err(err) => return handle_error(&err, "BlankCollection::new - can't open connection to db panya"),
    };

    let res = refresh_coll.find_latests("refresh_frequency", None, 10, None)
        .await
        .unwrap_or(vec![]);

    if res.is_empty() {
        return Ok("ok".to_string())
    }
    Ok("ok".to_string())
}