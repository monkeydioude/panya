use chrono::Utc;

use crate::db::{
    entities::Timer,
    model::{BlankCollection, CollectionModel},
};

pub type Timers<'a> = BlankCollection<'a, Timer>;

impl<'a> Timers<'a> {
    pub async fn insert_one(&self, channel: &str) -> Option<()> {
        self.insert_many(&[Timer {
            channel: channel.to_string(),
            update_date: Utc::now().timestamp(),
        }])
        .await
        .map_err(|err| {
            error!("could not insert in timers collection: {}", err);
            err
        })
        .ok()
        .and(Some(()))
    }
}