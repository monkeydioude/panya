use crate::error::Error;
use futures::StreamExt;
use mongodb::{bson::doc, results::InsertManyResult, Collection, Database, options};
use rocket::tokio;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, vec};

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

pub trait CollectionModel<T>
where T: Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned,
{
    async fn insert_many(&self, data: &[T]) -> Result<InsertManyResult, Error> {
        if data.is_empty() {
            return Error::to_result_string("empty input")?;
        }

        self.collection()
            .insert_many(data, None)
            .await
            .map_err(Error::from)     
    }

    async fn find_by_field_values(&self, data: &[T], field: &str) -> Vec<T> {
        let mut in_values = vec![];

        for item in data {
            in_values.push(item.sort_by_value());
        }

        let filter = doc! {
            field: { "$in": in_values }
        };

        let mut cursor = match self.collection().find(filter, None).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut results = vec![];
        while let Some(Ok(res)) = cursor.next().await {
            results.push(res);
        }
        results
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

        let find_options = options::FindOptions::builder()
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
