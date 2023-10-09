use super::{bakery::PotentialArticle, vec::RemoveExisting};
use crate::{
    db::{
        channels::Channels,
        entities::Timer,
        model::{BlankCollection, CollectionModel, SortOrder},
    },
    utils::now_minus_minutes,
};
use mongodb::bson::doc;
use rocket::error;

pub async fn process_data_and_fetch_items(
    channels: &Channels<'_, PotentialArticle>,
    data: &Vec<PotentialArticle>,
    limit: i64,
) -> Vec<PotentialArticle> {
    let found: Vec<PotentialArticle> = channels.find_by_field_values(data, "link", limit).await;
    let to_insert = data.remove_existing(&found);
    if to_insert.is_empty() {
        return found.clone();
    }

    if let Err(err) = channels.insert_many(&to_insert).await {
        error!("{}", err);
        return vec![];
    }

    data.iter().take(limit as usize).cloned().collect()
}

pub async fn should_fetch_cookies(
    timers: &BlankCollection<'_, Timer>,
    channel: &str,
    cooldown: i64,
) -> bool {
    timers
        .find(
            doc! {
                "channel": channel,
                "update_date": {
                    "$gt": now_minus_minutes(cooldown),
                },
            },
            Some("update_date"),
            1,
            SortOrder::DESC,
        )
        .await
        .and_then(|v| {
            if v.is_empty() {
                return None;
            }
            Some(())
        })
        .is_none()
}
