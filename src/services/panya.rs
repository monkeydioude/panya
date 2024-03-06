use super::{bakery::PotentialArticle, vec::RemoveReplaceExisting};
use crate::{
    db::{
        channels::Channels,
        entities::Timer,
        model::{BlankCollection, CollectionModel, SortOrder},
    },
    utils::now_minus_minutes,
};
use mongodb::bson::doc;

pub async fn process_data_and_fetch_items(
    parsed_from_bakery: &Vec<PotentialArticle>,
    channels_coll: Channels<'_, PotentialArticle>,
    limit: i64,
) -> Vec<PotentialArticle> {
    // find existing links
    let existing_links = channels_coll.find_by_field_values(&parsed_from_bakery, "link", 0).await;
    // picks out existing links in db
    let to_insert = parsed_from_bakery.remove_existing(&existing_links);
    // nothing to insert, move on
    if !to_insert.is_empty() {
        let _ = channels_coll.insert_many(&to_insert).await;
    }
    // returns the wanted number of items
    parsed_from_bakery
        .replace_existing(&existing_links)
        .iter()
        .take(limit as usize)
        .cloned()
        .collect()
}

pub async fn should_fetch_items(
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
            Some("create_date"),
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
