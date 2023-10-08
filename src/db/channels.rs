use super::{mongo::Handle, model::{FieldSort, CollectionModel}};
use crate::error::Error;
use mongodb::{bson::doc, results::InsertManyResult, Collection, Database, IndexModel};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Channels<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    url: &'a str,
    db_name: &'a str,
}

impl<'a, T> Channels<'a, T>
where T: Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned,
{
    pub async fn insert_many(&self, data: &[T]) -> Result<InsertManyResult, Error> {
        // Works cause we dont store result, nor do we return it.
        // An Err() is returned, if that's the case.
        self.collection()
            // Oftenly creating new collectionm therefore index
            .create_index(IndexModel::builder().keys(doc! {"link": -1}).build(), None)
            .await?;
        CollectionModel::<T>::insert_many(self, data).await
    }

    pub fn get_database_name(&self) -> String {
        self.db_name.to_string()
    }

    pub fn new(url: &'a str, handle: &'a Handle, db_name: &'a str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Err(Error("no database found".to_string())),
        })
        .collection::<T>(url);
        Ok(Channels {
            db_name,
            url,
            handle,
            collection,
        })
    }
}


impl<'a, T> CollectionModel<T> for Channels<'a, T>
where
    T: Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned,
{

    fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    fn get_collection_name(&self) -> String {
        self.url.to_string()
    }

    fn get_database(&self) -> Option<&Database> {
        self.handle.database(self.db_name)
    }
}
