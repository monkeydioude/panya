use crate::error::Error;
use futures::{future, StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, Bson, Document},
    options::{FindOneAndUpdateOptions, FindOptions},
    results::InsertManyResult,
    Collection, Database,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{any::Any, collections::HashMap, fmt::Debug, vec};

use super::mongo::Handle;

#[derive(Copy, Clone, Debug)]
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

pub trait CollectionModelConstraint:
    Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned
{
}
impl<T> CollectionModelConstraint for T where
    T: Serialize + FieldSort<String> + Debug + Unpin + Send + Sync + DeserializeOwned
{
}

#[derive(Debug, Serialize, Deserialize)]
struct Counter {
    _id: String,
    seq: i32,
}

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
                FindOptions::builder()
                    .limit(limit)
                    .sort(doc! {"_id": SortOrder::DESC.value()})
                    .build(),
            )
            .await
        {
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
                field.unwrap_or("_id"): sort.into().unwrap_or(SortOrder::DESC).value(),
            })
            .build();

        match self
            .collection()
            .find(doc, find_options)
            .await
            .map_err(|err| {
                warn!(
                    "model::CollectionModel::find_latests could not find latest: {}",
                    err
                );
                err
            })
            .ok()
        {
            Some(mut cursor) => {
                let mut results = vec![];
                while let Some(Ok(res)) = cursor.next().await {
                    results.push(res);
                }
                Some(results)
            }
            None => None,
        }
    }

    async fn find_with_limits<L: PartialEq + Ord>(
        &self,
        field: &str,
        values: Vec<Bson>,
        limits: Option<HashMap<L, i64>>,
        mut max_limit: i64,
        sort_tuple: impl Into<Option<(&str, SortOrder)>>,
    ) -> Option<Vec<T>> {
        if limits.is_some() {
            max_limit = match limits.unwrap_or_default()
                .iter()
                .max() {
                    Some(res) => *res.1,
                    None => max_limit,
                };
        }
        let mut pipeline = vec![
            doc! { "$match": { field: { "$in": values } } },  // Step 1: Filter documents
            // doc! { "$sort": { "some_field": 1 } },  // Optional: sort by some field if needed
            doc! { "$group": {  // Step 2: Group by channel_id
                "_id": "$".to_owned()+field,
                "docs": { "$push": "$$ROOT" }  // Collect all documents per channel_id
            }},
            doc! { "$project": {  // Step 3: Limit the number of docs per group
                "docs": { "$slice": ["$docs", max_limit] }  // Limit number to 2 per group
            }}
        ];

        let sort_into = sort_tuple.into();
        if sort_into.is_some() {
            let usort_tuple = sort_into.unwrap();
            pipeline.insert(1, doc! { "$sort": {usort_tuple.0: usort_tuple.1.value() }});
        }
    
        let mut cursor = self.collection()
            .aggregate(pipeline, None)
            .await
            .map_err(|err| {
                warn!( "model::CollectionModel::find_with_limits could not find latest: {}", err);
                err
            })
            .ok()?;
            let mut results = Vec::<T>::new();
            while let Some(doc) = cursor.next().await {
                match doc {
                    Ok(doc) => match mongodb::bson::from_document::<T>(doc) {
                        Ok(t) => {
                            results.push(t)
                        },
                        Err(e) => {
                            warn!("model::CollectionModel::find_with_limits failed to deserialize document: {}", e);
                        },
                    },
                    Err(e) => {
                        warn!("model::CollectionModel::find_with_limits failed to retrieve document: {}", e);
                    },
                }
            }
            Some(results)
        
    }

    async fn find_latests(
        &self,
        field: &str,
        after: impl Into<Option<i64>>,
        limit: impl Into<Option<i64>>,
        sort: impl Into<Option<SortOrder>>,
        filter: impl Into<Option<Document>>,
    ) -> Option<Vec<T>> {
        if field.is_empty() {
            return None;
        }
        let find_options = FindOptions::builder()
            .limit(limit)
            .sort(doc! {
                field: sort.into().unwrap_or(SortOrder::DESC).value(),
            })
            .build();
        let mut filter_options = match filter.into() {
            Some(d) => d,
            None => doc!{},
        };
        let after_into = after.into();
        if after_into.is_some() {
            filter_options.insert(field, doc! {
                "$gt": after_into.unwrap(),
            });
        }

        self
            .collection()
            .find(filter_options, find_options)
            .await
            .map_err(|err| {
                warn!( "model::CollectionModel::find_latests could not find latest: {}", err);
                err
            })
            .ok()?
            .try_collect()
            .await
            .map_err(|err| {
                warn!( "model::CollectionModel::find_latests could collect: {}", err);
                err
            })
            .ok()
    }

    fn collection(&self) -> &Collection<T>;
    fn get_collection_name(&self) -> String;
    fn get_database(&self) -> Option<&Database>;
    fn get_diff_collection<C>(&self, coll: &str) -> Option<Collection<C>> {
        Some(self.get_database()?.collection::<C>(coll))
    }
    async fn get_next_seq(&self) -> mongodb::error::Result<i32> {
        let counters = self.get_diff_collection::<Counter>("counters")
            .ok_or(mongodb::error::Error::from(std::io::ErrorKind::NotFound))?;
        let filter = doc! { "_id": self.get_collection_name() };
        let update = doc! {
            "$inc": { "seq": 1 }, 
            "$setOnInsert": { "_id": self.get_collection_name() }
        };
        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(mongodb::options::ReturnDocument::After)
            .build();

        let result_doc = counters
            .find_one_and_update(filter, update, options)
            .await?
            .ok_or_else(|| mongodb::error::Error::from(std::io::ErrorKind::NotFound));

        result_doc.map(|doc| doc.seq)
    }

    async fn get_seq(&self, id: &str) -> mongodb::error::Result<i32> {
        let counters = self.get_diff_collection::<Counter>("counters")
            .ok_or(mongodb::error::Error::from(std::io::ErrorKind::NotFound))?;
        let res = counters
            .find_one(doc! {"_id": id}, None)
            .await?;
        match res {
            Some(r) => Ok(r.seq),
            None => self.get_next_seq().await,
        }
    }
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
        self.collection.name().to_string()
    }

    fn get_database(&self) -> Option<&Database> {
        self.handle.database(self.db_name)
    }
}

impl<'a, T: CollectionModelConstraint> BlankCollection<'a, T> {
    pub fn new(handle: &'a Handle, db_name: &'a str, collection_name: &str) -> Result<Self, Error> {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use crate::{config, db::{self, items::Items, model::{CollectionModel, SortOrder}, mongo::i32_to_bson}, entities::potential_articles::PotentialArticle};
    use super::{BlankCollection, FieldSort};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Test {
        link: String,
    }
    impl FieldSort<String> for Test {
        fn sort_by_value(&self) -> String {
            "pog".to_string()
        }
    }
    
    #[rocket::async_test]
    async fn test_get_seq() {
        let settings = config::Settings::new().unwrap();
        let db_handle = db::mongo::get_handle(&settings).await;
        let coll = BlankCollection::<Test>::new(&db_handle, "panya", "test").unwrap();
        // coll.insert_many(&[Test{}]).await.unwrap();

        println!("next sequence: {}", coll.get_next_seq().await.unwrap() );
    }

    #[rocket::async_test]
    async fn test_find_with_limits() {
        let settings = config::Settings::new().unwrap();
        let db_handle = db::mongo::get_handle(&settings).await;
        let coll = Items::<PotentialArticle>::new(&db_handle, "panya").unwrap();
        // coll.insert_many(&[Test{}]).await.unwrap();

        println!("next find_with_limits: {:?}", coll.find_with_limits(
			"channel_id",
			i32_to_bson(&vec![1, 2]), 
			Some(HashMap::from([
                (1, 10),
                (2, 5),
            ])),
			10,
			("create_date", SortOrder::DESC),
        ).await.unwrap());
    }
}
