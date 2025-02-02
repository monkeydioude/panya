use std::sync::Arc;

use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::db::channel::Channels;
use crate::entities::channel::Channel;
use crate::error::Error;
use crate::response::HTTPResponse;
use crate::services::grpc::{user_login, user_signup};
use crate::services::token::extract_auth;
use crate::{entities::user::User, error::HTTPError, request_guards::xqueryid::XQueryID};

use crate::db::items::Items;
use crate::db::model::{CollectionModel, SortOrder};
use crate::entities::potential_articles::PotentialArticle;
use crate::services::feed::GetFeedQuery;
use crate::utils::now_timestamp_ms;
use crate::{config::Settings, db::mongo::Handle};
use chrono::{Duration, Utc};
use mongodb::bson::doc;

use super::public_entities::public_channel::PublicChannel;
#[derive(Deserialize, Serialize, Debug)]
pub struct UserPayload {
    pub login: String,
    pub password: String,
    pub realm: String,
}

// /panya
#[post("/user/signup", format = "json", data = "<add_user>")]
pub async fn add_user(
    cookies: &CookieJar<'_>,
    add_user: Json<UserPayload>,
    settings: &rocket::State<Settings>,
    uuid: XQueryID,
) -> Result<Json<HTTPResponse>, HTTPError> {
    println!(
        "[INFO] ({}) User signup attempt with login {}",
        uuid, add_user.login
    );
    let resp = match user_signup(&settings.identity_server_addr, &add_user).await {
        Ok(r) => r,
        Err(err) => return Err(HTTPError::InternalServerError(err.into())),
    };

    if resp.code == 400 {
        return Err(HTTPError::BadRequest(Error(
            "login already used".to_string(),
        )));
    }
    if resp.code != 200 {
        return Err(HTTPError::InternalServerError(Error(
            "error during user signup".to_string(),
        )));
    }

    // let login_resp = match user_login(&settings.identity_server_addr, &add_user).await {
    //     Ok(r) => r,
    //     Err(err) => return Err(HTTPError::InternalServerError(err.into())),
    // };

    // cookies.add(
    //     Cookie::build(("Authorization", extract_auth(&login_resp.1)))
    //         .path("/") // Cookie is valid across the entire application
    //         .http_only(true) // Prevent access to the cookie via JavaScript
    //         .secure(true) // Send cookie only over HTTPS
    //         .max_age(rocket::time::Duration::hours(1)), // Set the cookie expiration
    // );

    return Ok(Json(HTTPResponse::created()));
}

#[post("/user/login", format = "json", data = "<login_user>")]
pub async fn login_user(
    cookies: &CookieJar<'_>,
    login_user: Json<UserPayload>,
    settings: &rocket::State<Settings>,
    uuid: XQueryID,
) -> Result<Json<HTTPResponse>, HTTPError> {
    println!(
        "[INFO] ({}) User signup attempt with login {}",
        uuid, login_user.login
    );
    let (resp, headers) = match user_login(&settings.identity_server_addr, &login_user).await {
        Ok(r) => r,
        Err(err) => return Err(HTTPError::InternalServerError(err.into())),
    };

    if resp.code == 404 {
        return Err(HTTPError::BadRequest(Error(
            "login does not exist".to_string(),
        )));
    }
    if resp.code != 200 {
        return Err(HTTPError::InternalServerError(Error(
            "error during user login".to_string(),
        )));
    }

    cookies.add(
        Cookie::build(("Authorization", extract_auth(&headers)))
            .path("/") // Cookie is valid across the entire application
            .http_only(true) // Prevent access to the cookie via JavaScript
            .secure(true) // Send cookie only over HTTPS
            .max_age(rocket::time::Duration::hours(1)), // Set the cookie expiration
    );

    return Ok(Json(HTTPResponse::ok()));
}

// GET /panya/user
#[get("/user")]
pub async fn show_user_channels(_uuid: XQueryID, user: User) -> Result<Json<User>, HTTPError> {
    Ok(Json(user))
}

// GET /panya/user/channels
#[get("/user/channels")]
pub async fn show_user(
    db_handle: &rocket::State<Arc<Handle>>,
    _uuid: XQueryID,
    user: User,
) -> Result<Json<Vec<PublicChannel>>, HTTPError> {
    let channels_coll = Channels::<Channel>::new(db_handle, "panya")?;
    let channels = channels_coll
        .find(doc! {"id": {"$in": &user.channel_ids}}, None, None, None)
        .await
        .unwrap_or_default();
    Ok(Json(PublicChannel::from_channels(channels)))
}

#[get("/user/feed?<query..>")]
pub async fn show_user_feed(
    db_handle: &rocket::State<Arc<Handle>>,
    settings: &rocket::State<Settings>,
    query: GetFeedQuery,
    xquery_id: XQueryID,
    user: User,
) -> Result<Json<Vec<PotentialArticle>>, HTTPError> {
    let items_coll = Items::<PotentialArticle>::new(db_handle, "panya")?;
    let max_limit = settings.default_item_per_feed;
    let time_bfore = now_timestamp_ms();
    let mut items = items_coll
        .find_with_limits(
            "channel_id",
            user.channel_ids,
            query.limits.unwrap_or_default(),
            max_limit,
            ("create_date", SortOrder::DESC),
            vec![(
                "create_date",
                doc! {"$gte": (Utc::now() - Duration::weeks(1)).timestamp_millis() },
            )],
        )
        .await
        .unwrap_or_else(|| vec![]);
    info!(
        "({}): time for query: {}ms",
        xquery_id,
        now_timestamp_ms() - time_bfore
    );
    items.sort_by(|a, b| b.cmp(a));
    Ok(Json(items))
}
