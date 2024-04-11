use super::model::FieldSort;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    pub channel: String,
    pub update_date: i64,
}

impl FieldSort<String> for Timer {
    fn sort_by_value(&self) -> String {
        self.channel.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Refresh {
    pub url: String,
    pub last_refresh_date: i64,
    // ms
    pub refresh_frequency: i64,
    pub update_date: i64,
    pub fetch_avg: f32,
    pub limit: i32,
}

impl FieldSort<String> for Refresh {
    fn sort_by_value(&self) -> String {
        self.last_refresh_date.to_string()
    }
}


#[derive(Deserialize)]
pub enum AscDesc {
    ASC,
    DESC,
}

impl AscDesc {
    pub fn as_str(&self) -> &'static str {
        match self {
            AscDesc::ASC => "ASC",
            AscDesc::DESC => "DESC"
        }
    }
}