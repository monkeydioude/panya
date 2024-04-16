use crate::{
    db::{
        entities::Timer, items::{get_channel_id, Items}, model::{BlankCollection, CollectionModel, SortOrder}
    }, entities::potential_articles::PotentialArticle, services::vec::RemoveReplaceExisting, utils::now_minus_minutes
};
use mongodb::bson::doc;

pub async fn process_data_and_fetch_items(
    articles: &Vec<PotentialArticle>,
    items_coll: Items<'_, PotentialArticle>,
    channel_name: &str,
    limit: i64,
) -> Vec<PotentialArticle> {
    // find existing links
    let existing_links = items_coll.find_by_field_values(&articles, "link", 0).await;
    // picks out existing links in db
    let mut to_insert = articles.remove_existing(&existing_links);

    // nothing to insert, move on
    if !to_insert.is_empty() {
        let channel_id = match get_channel_id(&items_coll, channel_name).await {
            Ok(r) => r,
            Err(_) => {
                error!("could not find any channel_id for: {}", channel_name);
                return vec![];
            },
        };
        to_insert
            .iter_mut()
            .for_each(|pa| {
                pa.channel_name = Some(channel_name.to_string());
                pa.channel_id = Some(channel_id);
            });
        let _ = items_coll.insert_many(&to_insert, None).await;
    }
    // returns the wanted number of items
    articles
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
