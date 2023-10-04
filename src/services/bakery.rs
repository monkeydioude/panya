use rocket::error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PotentialArticle {
    link: String,
    img: String,
    desc: String,
    date: u64,
}

pub async fn get_cookies_from_bakery(api_path: &str, url: &str) -> Option<Vec<PotentialArticle>> {
    let response = reqwest::get(format!("{}/bakery?url={}", api_path, url))
        .await;

    let raw_data = match response {
        Ok(res) => res.text().await.unwrap_or("[]".to_string()),
        Err(err) => {
            error!("{}", err);
            return None
        },
    };

    Some(serde_json::from_str::<Vec<PotentialArticle>>(&raw_data)
        .unwrap_or_else(|err| {
            error!("could not deserialize bakery response{}", err);
            vec![]
        }))
}