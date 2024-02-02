use std::error::Error;

use crate::db::channels::Channels;
use crate::db::entities::Timer;
use crate::db::model::{BlankCollection, CollectionModel, SortOrder};
use crate::services::bakery::{self, PotentialArticle};
use crate::services::cook_rss::cook;
use crate::services::panya::{process_data_and_fetch_items, should_fetch_items};
use crate::utils::clean_url;
use crate::{config::Settings, db::mongo::Handle};
use rocket::response::content::RawXml;
use rocket::{error, warn};

#[derive(FromForm)]
pub struct GetUrlQuery {
    pub url: String,
    pub limit: Option<i64>,
}

fn handle_error(err: &dyn Error, msg: &str, url: &str)-> RawXml<String> {
    error!(
        "{}: {}",
        msg,
        err
    );
    RawXml(cook(url, url, vec![]))
}

async fn return_db_articles(title: &str, link: &str,limit: i64, channels_coll: &Channels<'_, PotentialArticle>) -> RawXml<String> {
    let latests: Vec<PotentialArticle> = channels_coll
        .find_latests("create_date", None, limit, SortOrder::DESC)
        .await
        .unwrap_or(vec![]);

    return RawXml(cook(link, title, latests));
}

// /panya?url=
#[get("/?<query..>")]
pub async fn get_url(
    handle: &rocket::State<Handle>,
    settings: &rocket::State<Settings>,
    query: GetUrlQuery,
) -> RawXml<String> {
    if query.url.is_empty() {
        warn!("handler::get_url - no url found");
        return RawXml(cook(&query.url, &query.url, vec![]));
    }

    let url = clean_url(&query.url).unwrap_or(query.url.clone());

    let limit = query.limit.unwrap_or(5);
    let timers_coll = match BlankCollection::<Timer>::new(handle, "panya", "timers") {
        Ok(c) => c,
        Err(err) => return handle_error(&err, "BlankCollection::new - can't open connection to db panya", &url),
    };

    let channels_coll = match Channels::<PotentialArticle>::new(&url, handle, "channels") {
        Ok(c) => c,
        Err(err) => return handle_error(&err, "Channels::new - can't open connection to db channels", &url),
    };
    if !should_fetch_items(&timers_coll, &url, settings.bakery_trigger_cooldown).await {
        return return_db_articles(&url, &query.url, limit, &channels_coll).await;
    }

    let parsed_from_bakery = bakery::get_cookies_from_bakery(&settings.api_path, &query.url)
        .await
        .unwrap_or_default();
    if parsed_from_bakery.is_empty() {
        warn!("bakery::get_cookies_from_bakery - no articles found");
        return RawXml(cook(&query.url, &query.url, vec![]));
    }

    timers_coll.insert_one(&url).await;
    RawXml(cook(
        &query.url,
        &url,
        process_data_and_fetch_items(&parsed_from_bakery, channels_coll, limit).await,
    ))
}
