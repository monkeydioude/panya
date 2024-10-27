use std::collections::HashMap;

use mongodb::bson::doc;

use crate::{
    db::{channel::Channels, model::CollectionModel, mongo::to_bson_vec},
    entities::channel::Channel,
};

#[derive(FromForm)]
pub struct GetFeedQuery {
    pub ids: String,
    pub limits: Option<HashMap<i32, i64>>,
}

impl GetFeedQuery {
    pub async fn get_ids(&self, channels_coll: &Channels<'_, Channel>) -> Vec<i32> {
        let ids: Vec<i32> = self
            .ids
            .split(",")
            // convert from String to u32
            .filter_map(|e| e.trim().parse::<i32>().ok())
            .collect();

        let filter = doc! {
            "id": {"$in": to_bson_vec(&ids)},
            "last_successful_refresh": {"$gt": 0},
        };
        let res: Vec<i32> = channels_coll
            .find(filter, None, None, None)
            .await
            .unwrap_or_default()
            .iter()
            .map(|channel| channel.id)
            .collect();

        ids.iter()
            .filter_map(|id| match res.contains(id) {
                true => Some(*id),
                false => None,
            })
            .collect()
    }
}
