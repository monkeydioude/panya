use std::collections::HashMap;

use mongodb::bson::doc;

#[derive(FromForm)]
pub struct GetFeedQuery {
    pub limits: Option<HashMap<i32, i64>>,
}
