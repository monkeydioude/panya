use crate::{
    db::{
        entities::Timer,
        items::Items,
        model::{BlankCollection, CollectionModel, SortOrder},
    },
    entities::potential_articles::PotentialArticle,
    utils::now_minus_minutes,
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
            doc! {"channel_name": url},
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
