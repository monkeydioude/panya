use crate::error::Error;
use futures::StreamExt;
use mongodb::{bson::{doc, Document}, results::InsertManyResult, Collection, Database, options::FindOptions};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, vec};

use super::mongo::Handle;

#[derive(Copy, Clone)]
pub enum SortOrder {
    ASC = 1,
    DESC = -1,
}

impl SortOrder {
    pub fn value(&self) -> i32 {
        *self as i32
    }
}

impl From<Option<SortOrder>> for SortOrder {
    fn from(value: Option<SortOrder>) -> Self {
        value.unwrap_or(SortOrder::ASC)
    }
}

pub trait CollectionModelConstraint : Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned {}
impl<T> CollectionModelConstraint for T
where T: Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned {}

pub trait CollectionModel<T: CollectionModelConstraint> {
    async fn insert_many(&self, data: &[T]) -> Result<InsertManyResult, Error> {
        if data.is_empty() {
            return Error::to_result_string("empty input")?;
        }

        self.collection()
            .insert_many(data, None)
            .await
            .map_err(Error::from)     
    }

    async fn find_by_field_values(&self, data: &[T], field: &str, limit: i64) -> Vec<T> {
        let mut in_values = vec![];
        for item in data {
            in_values.push(item.sort_by_value());
        }

        let filter = doc! {field: { "$in": in_values }};
        let mut cursor = match self
            .collection()
            .find(
                filter, 
                FindOptions::builder().limit(limit).sort(doc! {"_id": SortOrder::DESC.value()}).build(),
            ).await {
                Ok(c) => c,
                Err(_) => return vec![],
        };
        let mut results = vec![];
        while let Some(Ok(res)) = cursor.next().await {
            results.push(res);
        }

        results
    }

    async fn find(
        &self,
        doc: Document,
        field: Option<&str>,
        limit: impl Into<Option<i64>>,
        sort: impl Into<Option<SortOrder>>,
    ) -> Option<Vec<T>> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .sort(doc! {
                field.unwrap_or("_id"): sort.into().unwrap_or(SortOrder::ASC).value(),
            })
            .build();

            match self
            .collection()
            .find(doc, find_options)
            .await
            .map_err(|err| {
                warn!("model::CollectionModel::find_latests could not find latest: {}", err);
                err
            })
            .ok() {
                Some(mut cursor) => {
                    let mut results = vec![];
                    while let Some(Ok(res)) = cursor.next().await {
                        results.push(res);
                    }
                    Some(results)
                },
                None => None,
            }
    }

    async fn find_latests(
        &self,
        field: &str,
        after: impl Into<Option<i64>>,
        limit: impl Into<Option<i64>>,
        sort: impl Into<Option<SortOrder>>,
    ) -> Option<Vec<T>> {
        if field.is_empty() {
            return None
        }

        let find_options = FindOptions::builder()
            .limit(limit)
            .sort(doc! {
                field: sort.into().unwrap_or(SortOrder::ASC).value(),
            })
            .build();
        let mut doc = doc! {};
        let after_into = after.into();

        if !after_into.is_none() {
            doc = doc! {
                field: {
                    "$gt": after_into.unwrap(),
                },
            }
        }

        match self
            .collection()
            .find(doc, find_options)
            .await
            .map_err(|err| {
                warn!("model::CollectionModel::find_latests could not find latest: {}", err);
                err
            })
            .ok() {
                Some(mut cursor) => {
                    let mut results = vec![];
                    while let Some(Ok(res)) = cursor.next().await {
                        results.push(res);
                    }
                    Some(results)
                },
                None => None,
            }
    }

    fn collection(&self) -> &Collection<T>;
    fn get_collection_name(&self) -> String;
    fn get_database(&self) -> Option<&Database>;
}


pub trait FieldSort<V> {
    fn sort_by_value(&self) -> V;
}

pub struct BlankCollection<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    db_name: &'a str,
}

impl<'a, T: CollectionModelConstraint> CollectionModel<T> for BlankCollection<'a, T> {
    fn collection(&self) -> &Collection<T> {
        &self.collection
    }

    fn get_collection_name(&self) -> String {
        self.db_name.to_string()
    }

    fn get_database(&self) -> Option<&Database> {
        self.handle.database(self.db_name)
    }
}

impl <'a, T: CollectionModelConstraint> BlankCollection<'a, T> {
    pub fn new(
        handle: &'a Handle,
        db_name: &'a str,
        collection_name: &str,
    ) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
                Some(res) => res,
                None => return Error::to_result_string("no database found"),
            })
            .collection::<T>(collection_name);

        Ok(Self {
            db_name,
            handle,
            collection,
        })
    }
}