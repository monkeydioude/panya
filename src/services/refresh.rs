use chrono::Utc;

use crate::db::{
    entities::Refresh,
    model::{BlankCollection, CollectionModel},
};

pub type Refresher<'a> = BlankCollection<'a, Refresh>;

impl<'a> Refresher<'a> {
}
