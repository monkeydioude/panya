use crate::{
    db::{
        channel::{get_channel_id, Channels}, entities::Timer, items::Items, model::{BlankCollection, CollectionModel, SortOrder}
    }, entities::{channel::{Channel, SourceType}, potential_articles::PotentialArticle}, services::vec::RemoveReplaceExisting, utils::now_minus_minutes
};
use mongodb::bson::doc;

/// return_db_articles fetch a `limit` amount of items from db,
/// then turn them into XML.
pub async fn return_db_articles(
    url: &str,
    limit: i64,
    items_coll: &Items<'_, PotentialArticle>,
) -> Vec<PotentialArticle> {
    items_coll
        .find_latests(
            "create_date", 
            None, 
            limit, 
            SortOrder::DESC,
            doc! {"channel_name": url}
        )
        .await
        .unwrap_or(vec![])
}

/// process_data_and_fetch_items compares fetched articles from bakery against existing ones in DB,
/// then insert those not existing and then returns the latest `limit` number of articles.
pub async fn process_data_and_fetch_items(
    articles: &Vec<PotentialArticle>,
    items_coll: Items<'_, PotentialArticle>,
    channels_coll: Channels<'_, Channel>,
    channel_name: &str,
    limit: i64,
) -> Vec<PotentialArticle> {
    // find existing links
    let existing_links = items_coll.find_by_field_values(&articles, "link", 0).await;
    // picks out existing links in db
    let mut to_insert = articles.remove_existing(&existing_links);


    // something to insert
    if !to_insert.is_empty() {
        let channel_id = match get_channel_id(&channels_coll, channel_name, SourceType::Bakery).await {
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
    items_coll
        .find_latests(
            "create_date", 
            None, 
            limit, 
            SortOrder::DESC,
            doc! {"channel_name": channel_name}
        )
        .await
        .unwrap_or(vec![])
}

/// should_fetch_items assert if items should be fetched or not,
/// with respect to the update date of the 
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
