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
