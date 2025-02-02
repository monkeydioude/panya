use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::db::model::{FieldSort, PrimaryID};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd)]
pub struct PotentialArticle {
    pub link: String,
    pub img: String,
    pub title: Option<String>,
    #[serde(rename(serialize = "description"))]
    pub desc: String,
    #[serde(alias = "date", rename(serialize = "pubDate"))]
    pub create_date: i64,
    #[serde(rename(serialize = "channelTitle"))]
    pub channel_name: Option<String>,
    // #[serde(skip_serializing)]
    pub channel_id: Option<i32>,
    pub categories: Option<Vec<String>>,
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
        Utc.timestamp_opt(
            self.create_date / 1000,
            self.create_date as u32 % 1000 * 1000000,
        )
        .single()
        .unwrap_or(Utc::now())
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
    }

    pub fn some_human_date(&self) -> Option<String> {
        Some(self.human_date())
    }
}

impl Ord for PotentialArticle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.create_date.cmp(&other.create_date)
    }
}

impl PrimaryID<i32> for PotentialArticle {
    fn get_primary_id(&self) -> Option<i32> {
        self.channel_id
    }
}

impl FieldSort<String> for PotentialArticle {
    fn sort_by_value(&self) -> String {
        self.link.clone()
    }
}
