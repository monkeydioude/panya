use rocket::serde::json::Json;

use crate::db::channel::Channels;
use crate::entities::channel::Channel;
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

// GET /panya/user
#[get("/user")]
pub async fn show_user_channels(_uuid: XQueryID, user: User) -> Result<Json<User>, HTTPError> {
    Ok(Json(user))
}

// GET /panya/user/channels
#[get("/user/channels")]
pub async fn show_user(
    db_handle: &rocket::State<Handle>,
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
    db_handle: &rocket::State<Handle>,
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
