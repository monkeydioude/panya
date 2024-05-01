use std::error::Error as StdError;

use crate::db::channel::Channels;
use crate::db::items::Items;
use crate::db::model::CollectionModel;
use crate::entities::channel::{new_with_seq_db, SourceType};
use crate::entities::potential_articles::PotentialArticle;
use crate::services::cook_rss::cook;
use crate::services::panya::return_db_articles;
use crate::utils::clean_url;
use crate::db::mongo::Handle;
use crate::error::Error;
use mongodb::bson::doc;
use rocket::response::content::RawXml;
use rocket::serde::json::Json;
use rocket::{error, warn};
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct GetUrlQuery {
    url: String,
    limit: Option<i64>,
}

fn handle_error(err: &dyn StdError, msg: &str, url: &str)-> RawXml<String> {
    error!(
        "{}: {}",
        msg,
        err
    );
    RawXml(cook(url, url, vec![]))
}

// /panya?url=
#[get("/?<query..>")]
pub async fn get_url(
    handle: &rocket::State<Handle>,
    query: GetUrlQuery,
) -> RawXml<String> {
    if query.url.is_empty() {
        warn!("handler::get_url - no url found");
        return RawXml(cook(&query.url, &query.url, vec![]));
    }
    let url = clean_url(&query.url).unwrap_or(query.url.clone());
    let limit = query.limit.unwrap_or(5);
    let items_coll = match Items::<PotentialArticle>::new(handle, "panya") {
        Ok(c) => c,
        Err(err) => return handle_error(&err, "Items::new - can't open connection to db panya", &url),
    };
    let channels_coll = match Channels::new(handle, "panya") {
        Ok(c) => c,
        Err(err) => return handle_error(&err, "Channels::new - can't open connection to db panya", &url),
    };
    let items = return_db_articles(&url, limit, &items_coll).await;
    // this is temporary
    if items.is_empty() {
        if let Err(err) = new_with_seq_db(
            &url,
            SourceType::Bakery,
            &channels_coll,
        ).await {
            eprintln!("{}", err);
        }
    }
    RawXml(cook(&url, &query.url, items))
}

#[derive(Deserialize, Serialize)]
pub struct AddChannel {
    channel_name: String,
    source_type: Option<SourceType>,
    #[serde(skip_deserializing)]
    channel_id: i32,
}

// /panya/channel
#[post("/channel", format="json", data="<add_channel>")]
pub async fn add_url(
    handle: &rocket::State<Handle>,
    add_channel: Json<AddChannel>,
) -> Result<Json<AddChannel>, Error> {
    let channels_coll = Channels::new(handle, "panya")?;
    let source_type = add_channel.source_type.unwrap_or(SourceType::Other);
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