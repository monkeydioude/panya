use super::mongo::Handle;
use crate::error::Error;
use futures::StreamExt;
use mongodb::{bson::doc, results::InsertManyResult, Collection, Database, IndexModel};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, vec};

#[derive(Debug)]
pub struct Channels<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    url: &'a str,
}

pub trait FieldSort<V> {
    fn sort_by_value(&self) -> V;
}

impl<'a, T> Channels<'a, T>
where
    T: Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned,
{
    pub async fn insert_many(&self, data: &[T]) -> Result<InsertManyResult, Error> {
        if data.is_empty() {
            return Error::to_result_string("empty input")?;
        }

        // Works cause we dont store result, nor do we return it.
        // An Err() is returned, if that's the case.
        self.collection()
            // Oftenly creating new collectionm therefore index
            .create_index(IndexModel::builder().keys(doc! {"link": -1}).build(), None)
            .await?;

        self.collection()
            .insert_many(data, None)
            .await
            .map_err(Error::from)
    }

    pub async fn find_by_field(&self, data: &[T], field: &str) -> Vec<T> {
        let mut in_values = vec![];

        for item in data {
            in_values.push(item.sort_by_value());
        }

        let filter = doc! {
            field: { "$in": in_values }
        };

        let mut cursor = match self.collection.find(filter, None).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut results = vec![];
        while let Some(Ok(res)) = cursor.next().await {
            results.push(res);
        }
        results
    }

    pub fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    pub fn get_collection_name(&self) -> String {
        self.url.to_string()
    }

    pub fn get_database(&self) -> Option<&Database> {
        self.handle.database(&Channels::<T>::get_database_name())
    }

    pub fn get_database_name() -> String {
        "channels".to_string()
    }

    pub fn new(url: &'a str, handle: &'a Handle) -> Result<Self, Error> {
        let collection = (match handle.database(&Channels::<T>::get_database_name()) {
            Some(res) => res,
            None => return Err(Error("no database found".to_string())),
        })
        .collection::<T>(url);
        Ok(Channels {
            url,
            handle,
            collection,
        })
    }
}
