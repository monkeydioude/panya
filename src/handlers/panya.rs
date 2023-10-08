use crate::db::channels::Channels;
use crate::db::model::{CollectionModel, SortOrder};
use crate::services::bakery::{self, PotentialArticle};
use crate::services::cook_rss::cook;
use crate::services::panya::{process_data_and_fetch_items, should_fetch_cookies};
use crate::{config::Settings, db::mongo::Handle};
use rocket::response::content::RawXml;
use rocket::{error, warn};

pub fn try_fixing_url(url: &str) -> String {
    println!("{}", url);
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string().clone();
    }
    "http://".to_string() + url
}

#[derive(FromForm)]
pub struct GetUrlQuery {
    url: String,
}

#[get("/?<query..>")]
pub async fn get_url(
    handle: &rocket::State<Handle>,
    settings: &rocket::State<Settings>,
    query: GetUrlQuery,
) -> RawXml<String> {
    if query.url.is_empty() {
        warn!("handler: get_url - no url found");
        return RawXml(cook(vec![]));
    }

    let channels = match Channels::<PotentialArticle>::new(&query.url, handle, "channels") {
        Ok(c) => c,
        Err(err) => {
            error!(
                "can't open connection to db {}: {}",
                "channels",
                err
            );
            return RawXml(cook(vec![]));
        }
    };

    if !should_fetch_cookies(&channels).await {
        let latests = channels.find_latests(
                "create_date", 
                None, 
                5, 
                SortOrder::DESC,
            ).await
            .unwrap_or(vec![]);

        return RawXml(cook(latests));
    }

    let data = bakery::get_cookies_from_bakery(&settings.api_path, &query.url)
        .await
        .unwrap_or_default();
    if data.is_empty() {
        warn!("no articles found");
        return RawXml(cook(vec![]));
    }

    RawXml(cook(process_data_and_fetch_items(&channels, &data).await))
}
