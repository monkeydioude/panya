use std::collections::HashMap;

use crate::db::items::Items;
use crate::db::model::{CollectionModel, SortOrder};
use crate::entities::potential_articles::PotentialArticle;
use crate::error::{Error, HTTPError};
use crate::services::grpc::jwt_status;
use crate::{config::Settings, db::mongo::Handle};
use mongodb::bson::doc;
use rocket::serde::json::Json;
use rocket::{error, warn};

use super::Token;

#[derive(FromForm)]
pub struct GetFeedQuery {
    pub ids: String,
    pub limits: Option<HashMap<i32, i64>>,
}

#[get("/feed?<query..>")]
pub async fn get_feed(
    db_handle: &rocket::State<Handle>,
    settings: &rocket::State<Settings>,
    query: GetFeedQuery,
    token: Token,
) -> Result<Json<Vec<PotentialArticle>>, HTTPError> {
    if let Err(err) = jwt_status(
        Box::leak(settings.identity_server_addr.clone().into_boxed_str()),
        &token.0,
    )
    .await
    {
        return Err(HTTPError::Unauthorized(Error::from(err)));
    }
    if query.ids.is_empty() {
        warn!("handler::get_channels - no ids found");
        return Ok(Json(vec![]));
    }
    let ids: Vec<i32> = query
        .ids
        .split(",")
        // convert from String to u32
        .filter_map(|e| e.trim().parse::<i32>().ok())
        .collect();
    let items_coll = match Items::<PotentialArticle>::new(db_handle, "panya") {
        Ok(c) => c,
        Err(err) => {
            error!("{}", err);
            return Ok(Json(vec![]));
        }
    };
    let max_limit = settings.default_item_per_feed;
    let mut items = items_coll
        .find_with_limits(
            "channel_id",
            ids,
            query.limits.unwrap_or_default(),
            max_limit,
            ("create_date", SortOrder::DESC),
        )
        .await
        .unwrap_or_else(|| vec![]);
    items.sort_by(|a, b| b.cmp(a));
    Ok(Json(items))
}

#[cfg(test)]
mod tests {
    use crate::{config, db};
    use rocket::local::asynchronous::Client;

    #[rocket::async_test]
    async fn test_get_channels() {
        let settings = config::Settings::new().unwrap();
        let rocket = rocket::build()
            .mount("/panya", routes![super::get_feed])
            .manage(db::mongo::get_handle(&settings).await)
            .manage(settings);
        let client = Client::untracked(rocket)
            .await
            .expect("valid rocket instance");

        // Dispatch a request to the '/hello' route
        let response = client.get("/panya/channels?ids=1,2,3").dispatch().await;
        println!("res: {}", response.into_string().await.unwrap());
        // Check the response status code and body
        // assert_eq!(response.status(), Status::Ok);
        // assert_eq!(response.into_string().await.unwrap(), "Hello, world!");
    }
}
