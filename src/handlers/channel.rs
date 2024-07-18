use mongodb::bson::doc;
use rocket::serde::json::Json;
use crate::entities::channel::Channel;
use crate::error::Error;

use crate::db::{channel::Channels, model::{CollectionModel, SortOrder}, mongo::Handle};

#[get("/channels")]
pub async fn get_channel_list(
    db_handle: &rocket::State<Handle>,
) -> Result<Json<Vec<Channel>>, Error> {
    let channels = Channels::new(db_handle, "panya")?
        .find(doc!{}, None, None, SortOrder::ASC).await
        .unwrap_or_default();
    Ok(Json(channels))
}