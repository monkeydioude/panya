use crate::db::items::Items;
use crate::db::model::{CollectionModel, SortOrder};
use crate::entities::potential_articles::PotentialArticle;
use crate::entities::user::User;
use crate::error::HTTPError;
use crate::request_guards::xqueryid::XQueryID;
use crate::services::feed::GetFeedQuery;
use crate::utils::now_timestamp_ms;
use crate::{config::Settings, db::mongo::Handle};
use chrono::{Duration, Utc};
use mongodb::bson::doc;
use rocket::serde::json::Json;

#[get("/feed?<query..>")]
pub async fn get_feed(
    db_handle: &rocket::State<Handle>,
    settings: &rocket::State<Settings>,
    query: GetFeedQuery,
    xquery_id: XQueryID,
    user: User,
) -> Result<Json<Vec<PotentialArticle>>, HTTPError> {
    let items_coll = Items::<PotentialArticle>::new(db_handle, "panya")?;
    let max_limit = settings.default_item_per_feed;
    let time_bfore = now_timestamp_ms();
    let mut items = items_coll
        .find_with_limits(
            "channel_id",
            user.channel_ids,
            query.limits.unwrap_or_default(),
            max_limit,
            ("create_date", SortOrder::DESC),
            vec![(
                "create_date",
                doc! {"$gte": (Utc::now() - Duration::weeks(1)).timestamp_millis() },
            )],
        )
        .await
        .unwrap_or_else(|| vec![]);
    info!(
        "({}): time for query: {}ms",
        xquery_id,
        now_timestamp_ms() - time_bfore
    );
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
