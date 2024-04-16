use super::{
    model::{CollectionModel, CollectionModelConstraint},
    mongo::Handle,
};
use crate::{entities::potential_articles::PotentialArticle, error::Error};
use mongodb::{bson::doc, results::InsertManyResult, Collection, Database, IndexModel};
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Items<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    db_name: String,
}

impl<'a, T: CollectionModelConstraint> Items<'a, T> {
    pub async fn insert_many(&self, data: &[T], index: Option<String>) -> Result<InsertManyResult, Error> {
        let idx = index.unwrap_or_else(|| "create_date".to_string());
        // Works cause we dont store result, nor do we return it.
        // An Err() is returned, if that's the case.
        self.collection()
            // Oftenly creating new collectionm therefore index
            .create_index(IndexModel::builder().keys(doc! {idx: -1}).build(), None)
            .await?;
        CollectionModel::<T>::insert_many(self, data).await
    }

    pub fn get_database_name(&self) -> &String {
        &self.db_name
    }

    pub fn new(handle: &'a Handle, db_name: &'a str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Err(Error("no database found".to_string())),
        })
        .collection::<T>("items");
        Ok(Items {
            db_name: db_name.to_string(),
            handle,
            collection,
        })
    }
}

impl<'a, T: CollectionModelConstraint> CollectionModel<T> for Items<'a, T> {
    fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    fn get_collection_name(&self) -> String {
        self.collection.name().to_string()
    }

    fn get_database(&self) -> Option<&Database> {
        self.handle.database(&self.db_name)
    }
}

pub async fn get_channel_id(items_coll: &Items<'_, PotentialArticle>, channel_name: &str) -> Result<i32, mongodb::error::Error> {
    match items_coll
        .find(doc!{"channel_name": channel_name}, None, 1, None)
        .await
        .unwrap_or_default()
        .pop() {
            Some(p) => p.channel_id
                .ok_or_else(|| mongodb::error::Error::from(std::io::ErrorKind::NotFound)),
            None => items_coll.get_next_seq().await,
        }
}