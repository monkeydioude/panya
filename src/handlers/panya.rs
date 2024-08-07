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
use mongodb::bson::doc;
use rocket::response::content::RawXml;
use rocket::{error, warn};

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
    // println!("{:?}", request);
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
    if items.is_empty() 
        && channels_coll.find_one("name", &url).await.is_none() {
        if let Err(err) = new_with_seq_db(
            &url,
            SourceType::Bakery,
            &channels_coll,
        ).await {
            eprintln!("{}", err);
        } else {
            
        }
    }
    RawXml(cook(&url, &query.url, items))
}