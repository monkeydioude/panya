use crate::db::channels::Channels;
use crate::db::entities::Timer;
use crate::db::model::{CollectionModel, SortOrder, BlankCollection};
use crate::services::bakery::{self, PotentialArticle};
use crate::services::cook_rss::cook;
use crate::services::panya::{process_data_and_fetch_items, should_fetch_cookies};
use crate::{config::Settings, db::mongo::Handle};
use rocket::response::content::RawXml;
use rocket::{error, warn};

#[derive(FromForm)]
pub struct GetUrlQuery {
    pub url: String,
    pub limit: Option<i64>,
}

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

    let limit = query.limit.unwrap_or(5);
    let timers = match BlankCollection::<Timer>::new(handle, "panya", "timers") {
        Ok(c) => c,
        Err(err) => {
            error!("BlankCollection::new - can't open connection to db panya: {}", err);
            return RawXml(cook(&query.url, &query.url, vec![]));
        },
    };
    let channels = match Channels::<PotentialArticle>::new(&query.url, handle, "channels") {
        Ok(c) => c,
        Err(err) => {
            error!("Channels::new - can't open connection to db channels: {}", err);
            return RawXml(cook(&query.url, &query.url, vec![]));
        }
    };
    if !should_fetch_cookies(&timers, &query.url).await {
        let latests = channels.find_latests(
                "_id", 
                None, 
                limit, 
                SortOrder::DESC,
            ).await
            .unwrap_or(vec![]);

        return RawXml(cook(&query.url, &query.url, latests));
    }

    let data = bakery::get_cookies_from_bakery(&settings.api_path, &query.url)
        .await
        .unwrap_or_default();
    if data.is_empty() {
        warn!("bakery::get_cookies_from_bakery - no articles found");
        return RawXml(cook(&query.url, &query.url, vec![]));
    }

    timers.insert_one(&query.url).await;
    RawXml(cook(&query.url, &query.url, process_data_and_fetch_items(&channels, &data, limit).await))
}
