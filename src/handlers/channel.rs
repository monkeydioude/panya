use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use crate::entities::channel::{new_with_seq_db, Channel, SourceType};
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

#[derive(Deserialize, Serialize)]
pub struct AddChannel {
    channel_name: String,
    source_type: Option<SourceType>,
    #[serde(skip_deserializing)]
    channel_id: i32,
}

async fn find_out_source_type(channel_name: &str) -> SourceType {
    let response = match reqwest::get(channel_name)
        .await {
            Ok(res) => res,
            Err(err) => {
                warn!("find_out_source_type: error: {}", err);
                return SourceType::Other;
            },
        };
    let content_type = match response.headers().get("content-type") {
        Some(header) => header.to_str().unwrap_or_default(),
        None => "",
    };
    if content_type.find("application/xml").is_some() {
        return SourceType::RSSFeed;
    }
    SourceType::Other
}

// /panya/channel
#[post("/channel", format="json", data="<add_channel>")]
pub async fn add_url(
    handle: &rocket::State<Handle>,
    add_channel: Json<AddChannel>,
) -> Result<Json<AddChannel>, Error> {
    let channels_coll = Channels::new(handle, "panya")?;
    let source_type = match add_channel.source_type {
        Some(res) => res,
        None => find_out_source_type(&add_channel.channel_name).await,
    };
    let mut channel_opt = channels_coll.find_one("name", &add_channel.channel_name).await;
    if channel_opt.is_none() {
        channel_opt = new_with_seq_db(
            &add_channel.channel_name,
            source_type,
            &channels_coll,
        ).await
        .ok();
    }

    channel_opt
        .ok_or_else(|| Error::string(&format!("error adding the channel: {}", add_channel.channel_name)))
        .and_then(|c| {
            Ok(Json(AddChannel {
                channel_name: c.name,
                source_type: Some(c.source_type),
                channel_id: c.id,
            }))
        })
}

// /panya/channel
#[delete("/channel/<id>")]
pub async fn delete_url(
    handle: &rocket::State<Handle>,
    id: i32,
) -> Result<String, Error> {
    let channels_coll = Channels::new(handle, "panya")?;
    channels_coll.delete_one("id", id)
        .await
        .map(|rr| rr.deleted_count.to_string())
}