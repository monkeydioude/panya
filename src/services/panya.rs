use crate::{db::{channels::Channels, model::CollectionModel}, utils::now_minus_minutes};
use rocket::error;

use super::{bakery::PotentialArticle, vec::RemoveExisting};

pub async fn process_data_and_fetch_items(
    channels: &Channels<'_, PotentialArticle>,
    data: &Vec<PotentialArticle>,
) -> Vec<PotentialArticle> {
    let found: Vec<PotentialArticle> = channels.find_by_field_values(data, "link").await;
    let to_insert = data.remove_existing(&found);
    if to_insert.is_empty() {
        return vec![];
    }

    if let Err(err) = channels.insert_many(&to_insert).await {
        error!("{}", err);
        return vec![];
    }

    data.clone()
}

pub async fn should_fetch_cookies(channels: &Channels<'_, PotentialArticle>) -> bool {
    channels.find_latests(
        "create_date", 
        now_minus_minutes(10), 
        1,
        None,
    )
    .await
    .and_then(|v| {
        if v.is_empty() {
            return None            
        }
        Some(())
    })
    .is_none()
}