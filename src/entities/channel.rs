use serde::Serialize;

use super::potential_articles::PotentialArticle;

#[derive(Serialize)]
pub struct Channel
{
    pub id: i32,
    pub channel: String,
    pub items: Vec<PotentialArticle>,
}