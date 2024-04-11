use rocket::error;

use crate::converters::string::to_articles;
use crate::db::model::FieldSort;
use crate::entities::potential_articles::PotentialArticle;


impl FieldSort<String> for PotentialArticle {
    fn sort_by_value(&self) -> String {
        self.link.clone()
    }
}

pub async fn get_cookies_from_bakery(api_path: &str, url: &str) -> Option<Vec<PotentialArticle>> {
    let response = reqwest::get(format!("{}/bakery?url={}", api_path, url)).await;
    let raw_data = match response {
        Ok(res) => res.text().await.unwrap_or("[]".to_string()),
        Err(err) => {
            error!("{}", err);
            return None;
        }
    };
    Some(to_articles(&raw_data))
}
