use chrono::{TimeZone, Utc};
use rocket::error;
use serde::{Deserialize, Serialize};

use crate::db::model::FieldSort;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PotentialArticle {
    pub link: String,
    pub img: String,
    pub desc: String,
    #[serde(alias = "date")]
    pub create_date: i64,
}

impl PotentialArticle {
    pub fn some_link(&self) -> Option<String> {
        Some(self.link.clone())
    }

    pub fn some_img(&self) -> Option<String> {
        Some(self.img.clone())
    }

    pub fn some_desc(&self) -> Option<String> {
        Some(self.desc.clone())
    }

    pub fn some_create_date(&self) -> Option<i64> {
        Some(self.create_date)
    }

    pub fn human_date(&self) -> String {
        Utc.timestamp_opt(self.create_date, 0)
            .single()
            .unwrap_or(Utc::now())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    pub fn some_human_date(&self) -> Option<String> {
        Some(self.human_date())
    }
}

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
    Some(
        serde_json::from_str::<Vec<PotentialArticle>>(&raw_data).unwrap_or_else(|err| {
            error!("could not deserialize bakery response{}", err);
            vec![]
        }),
    )
}
