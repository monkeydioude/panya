use rocket::{warn, error};
use rocket::serde::json::Json;

use crate::services::bakery;
use crate::{
    config::Settings,db::mongo::Handle,
};

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
) -> String {
    if query.url == "" {
        warn!("handler: get_url - no url found");
        return "none".to_string();
    }

    let data = bakery::get_cookies_from_bakery(&settings.api_path, &query.url).await.unwrap_or_else(|| vec![]);

    if data.is_empty() {
        warn!("no articles found");
        return "none".to_string();
    }
    return "ok".to_string();
}
