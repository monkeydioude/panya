use std::vec;
use mongodb::bson::doc;
use rocket::serde::json::Json;

use crate::{db::{items::Items, model::{CollectionModel, SortOrder}, mongo::Handle}, entities::potential_articles::PotentialArticle};

#[get("/list/<id>")]
pub async fn get_list(
    db_handle: &rocket::State<Handle>,
    id: String,
) -> Json<Vec<Vec<PotentialArticle>>> {
    let items_coll = match Items::<PotentialArticle>::new(db_handle, "panya") {
        Ok(c) => c,
        Err(err) => {
			error!("{}", err);
			return Json(vec![]);
		}
    };
    let mut res = vec![];
    let mut filter = doc!{};
    if id != "all" {
        filter = doc! {"channel_id": id.parse::<i32>().unwrap_or_default() };
    }
    let mut items = items_coll
        .find(filter, Some("channel_id"), None, SortOrder::DESC).await
        .unwrap_or_default();
    while items.len() > 0 {
        let id = match items.first() {
            Some(v) => v.channel_id,
            None => {
                items = items.drain(1..).collect();
                continue;
            },
        };

        let pos = items
            .iter()
            .position(|el| el.channel_id != id)
            .unwrap_or_else(|| items.len());
            
        let col: Vec<PotentialArticle> = items.drain(..pos).collect();
        res.push(col);
    }
    
    Json(res)
}