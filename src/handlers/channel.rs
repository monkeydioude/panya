use crate::config::Settings;
use crate::db::model::Updatable;
use crate::entities::channel::{new_with_seq_db, Channel, SourceType};
use crate::error::{Error, HTTPError};
use crate::services::channels::find_out_source_type;
use crate::services::grpc::jwt_status;
use crate::services::link_op::trim_link;
use mongodb::bson::doc;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::db::{
    channel::Channels,
    model::{CollectionModel, SortOrder},
    mongo::Handle,
};

use super::Token;

#[derive(Deserialize, Serialize)]
pub struct AddChannel {
    channel_url: String,
    #[serde(skip_deserializing)]
    channel_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type: Option<SourceType>,
    #[serde(skip_deserializing)]
    channel_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_frequency: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_type: Option<SourceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_frequency: Option<i32>,
}

impl Updatable<i32, Channel> for UpdateChannel {
    fn update(&self, entity: Channel) -> Channel {
        let mut new = entity.clone();

        if let Some(source_type) = self.source_type {
            new.source_type = source_type;
        }
        if let Some(refresh_frequency) = self.refresh_frequency {
            new.refresh_frequency = refresh_frequency;
        }
        if let Some(name) = &self.name {
            new.name = name.clone();
        }
        new
    }
}

#[get("/channels")]
pub async fn get_channel_list(
    db_handle: &rocket::State<Handle>,
    settings: &rocket::State<Settings>,
    token: Token,
) -> Result<Json<Vec<Channel>>, Error> {
    if let Err(err) = jwt_status(
        Box::leak(settings.identity_server_addr.clone().into_boxed_str()),
        &token.0,
    )
    .await
    {
        return Err(Error::from(err));
    }
    let channels = Channels::new(db_handle, "panya")?
        .find(doc! {}, None, None, SortOrder::ASC)
        .await
        .unwrap_or_default();
    Ok(Json(channels))
}

// /panya/channel
#[post("/channel", format = "json", data = "<add_channel>")]
pub async fn add_url(
    handle: &rocket::State<Handle>,
    add_channel: Json<AddChannel>,
    settings: &rocket::State<Settings>,
    token: Token,
) -> Result<Json<AddChannel>, HTTPError> {
    if let Err(err) = jwt_status(
        Box::leak(settings.identity_server_addr.clone().into_boxed_str()),
        &token.0,
    )
    .await
    {
        return Err(HTTPError::Unauthorized(Error::from(err)));
    }
    let channels_coll =
        Channels::new(handle, "panya").map_err(|err| HTTPError::InternalServerError(err))?;
    let channel_name = trim_link(&add_channel.channel_url);
    let mut channel_opt = channels_coll.find_one("name", &channel_name).await;
    let url = &add_channel.channel_url;
    let source_type = match add_channel.source_type {
        Some(res) => res,
        None => match find_out_source_type(url).await {
            Ok(res) => res,
            Err(err) => return Err(HTTPError::BadRequest(err)),
        },
    };

    if channel_opt.is_none() {
        channel_opt = new_with_seq_db(&channel_name, url, source_type, &channels_coll, &settings)
            .await
            .ok();
    }

    channel_opt
        .ok_or_else(|| {
            HTTPError::InternalServerError(Error::str(&format!(
                "error adding the channel: {}",
                add_channel.channel_url
            )))
        })
        .and_then(|c| {
            Ok(Json(AddChannel {
                channel_url: c.url,
                channel_name: c.name,
                source_type: Some(c.source_type),
                channel_id: c.id,
                refresh_frequency: None,
            }))
        })
}

// /panya/channel
#[put("/channel/<id>", format = "json", data = "<update_channel>")]
pub async fn update_channel(
    handle: &rocket::State<Handle>,
    id: i32,
    update_channel: Json<UpdateChannel>,
    settings: &rocket::State<Settings>,
    token: Token,
) -> Result<Json<Channel>, Error> {
    if let Err(err) = jwt_status(
        Box::leak(settings.identity_server_addr.clone().into_boxed_str()),
        &token.0,
    )
    .await
    {
        return Err(Error::from(err));
    }
    Channels::new(handle, "panya")?
        .update_one("id", id, update_channel.into_inner())
        .await
        .map(|entity| Json(entity))
}

// /panya/channel
#[get("/channel/<id>")]
pub async fn get_channel(
    handle: &rocket::State<Handle>,
    id: i32,
    settings: &rocket::State<Settings>,
    token: Token,
) -> Result<Json<Channel>, Error> {
    if let Err(err) = jwt_status(
        Box::leak(settings.identity_server_addr.clone().into_boxed_str()),
        &token.0,
    )
    .await
    {
        return Err(Error::from(err));
    }
    let channels_coll = Channels::new(handle, "panya")?;
    Ok(Json(
        channels_coll
            .find_one("id", id)
            .await
            .ok_or(Error("No channel found".to_string()))?,
    ))
}

// /panya/channel
#[delete("/channel/<id>")]
pub async fn delete_channel(
    handle: &rocket::State<Handle>,
    id: i32,
    settings: &rocket::State<Settings>,
    token: Token,
) -> Result<String, Error> {
    if let Err(err) = jwt_status(
        Box::leak(settings.identity_server_addr.clone().into_boxed_str()),
        &token.0,
    )
    .await
    {
        return Err(Error::from(err));
    }
    let channels_coll = Channels::new(handle, "panya")?;
    channels_coll
        .delete_one("id", id)
        .await
        .map(|rr| rr.deleted_count.to_string())
}
