use serde::{Deserialize, Serialize};
use chrono::{TimeZone, Utc};

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
