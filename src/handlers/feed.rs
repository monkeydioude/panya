use std::collections::BTreeMap;

use rocket::{response::content::{RawJson}, serde::{json::Json, Deserialize}};

use crate::{config::Settings, entities::potential_articles::PotentialArticle, services::{cook_rss::cook, request_rss::request_rss}};

#[derive(FromForm)]
struct GetUrlQuery {
    pub urls: Vec<String>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FeedsRequest {
    pub rss_urls: Option<Vec<String>>,
    pub html_to_rss_urls: Option<Vec<String>>,
    pub global_item_per_feed: Option<i32>,
    pub item_per_feed: Option<BTreeMap<String, i32>>,
}

#[get("/feed?<query..>", format = "json")]
pub async fn get_feed(
  settings: &rocket::State<Settings>,
  query: GetUrlQuery,
) -> Json<Vec<PotentialArticle>> {
  let item_per_feed = 10;
  if query.urls.is_empty() {
    warn!("handler::get_feed - no urls found");
    return Json(vec![]);
}
  request_rss(&query.urls, settings.default_item_per_feed, &None);
  // format!("Received: xml_urls = {:?}, html_to_xml_urls = {:?}, item_per_feed: {}", payload.rss_urls, payload.html_to_rss_urls, item_per_feed)
  Json(vec![])
}