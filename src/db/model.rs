use crate::error::Error;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, to_document, Bson, Document},
    options::{FindOneAndUpdateOptions, FindOptions},
    results::{DeleteResult, InsertManyResult},
    Collection, Database,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    vec,
};
use thiserror::Error;

use super::mongo::{db_not_found_err, to_bson_vec, Handle};

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

pub trait CollectionModelConstraint<P: PartialEq>:
    Serialize
    + FieldSort<String>
    + PrimaryID<P>
    + Debug
    + Unpin
    + Send
    + Sync
    + DeserializeOwned
    + Clone
{
}

impl<P: PartialEq, T> CollectionModelConstraint<P> for T where
    T: Serialize
        + FieldSort<String>
        + PrimaryID<P>
        + Debug
        + Unpin
        + Send
        + Sync
        + DeserializeOwned
        + Clone
{
}

pub trait Updatable<P: PartialEq, T: CollectionModelConstraint<P>> {
    fn update(&self, entity: T) -> T;
}

pub trait CloneEntity<T> {
    fn clone_entity(&self) -> T;
}

#[derive(Debug, Serialize, Deserialize)]
struct Counter {
    _id: String,
    seq: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocsWrapper<T> {
    docs: Vec<T>,
}

#[derive(Debug, Error)]
pub struct ModelError {
    err: Option<Error>,
    success: bool,
}

impl Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "success ({}), {}",
            self.success,
            self.err
                .clone()
                .unwrap_or_else(|| Error("no error".to_string()))
        )
    }
}

pub trait CollectionModel<P: PartialEq + Into<Bson> + Clone, T: CollectionModelConstraint<P>> {
    async fn delete_one(&self, field: &str, value: P) -> Result<DeleteResult, Error> {
        self.collection()
            .delete_one(doc! {field: value}, None)
            .await
            .map_err(Error::from)
    }
    async fn update_one<U: Updatable<P, T>>(
        &self,
        field: &str,
        value: P,
        updater: &U,
    ) -> Result<T, Error> {
        let entity = self
            .find_one(field, value.clone())
            .await
            .ok_or(Error::str("update_one: No result found"))?;

        let updated_entity = updater.update(entity);
        // Convert the updated entity to a BSON document
        let update_doc = to_document(&updated_entity).map_err(|e| {
            warn!("Failed to convert updated entity to document: {}", e);
            Error::str("Failed to convert entity to document")
        })?;
        self.collection()
            .update_one(doc! {field: value}, doc! {"$set": update_doc}, None)
            .await
            .map(|_| updated_entity)
            .map_err(Error::from)
    }
    /// insert_many inserts an array of documents into the collection
    async fn insert_many(&self, data: &[T]) -> Result<InsertManyResult, Error> {
        if data.is_empty() {
            return Error::str_to_result("empty input")?;
        }

        self.collection()
            .insert_many(data, None)
            .await
            .map_err(Error::from)
    }

    async fn update_one_or_insert<UC: Updatable<P, T> + CloneEntity<T>>(
        &self,
        field: &str,
        value: P,
        updater: &UC,
    ) -> (bool, Option<Error>) {
        if let Err(err) = self.update_one(field, value, updater).await {
            if let Err(err) = self.insert_many(&vec![updater.clone_entity()]).await {
                return (false, Some(err));
            }
            return (true, Some(err));
        }
        (true, None)
    }

    /// find_by_field_values fetch a `limit` number of documents matching a `field`
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

    async fn find_one<F: Sized + Into<Bson>>(&self, field: &str, value: F) -> Option<T> {
        self.find(doc! {field: value}, None, 1, None)
            .await
            .and_then(|res| res.first().cloned())
    }

    async fn find_by_field(&self, field: &str, value: impl Into<Bson>) -> Option<T> {
        self.find_one(field, value).await
    }
    /// find returns document matching a `doc`, sorting on a `field` using a `sort` order (SortOrder),
    /// limited to a `limit` number of documents.
    async fn find(
        &self,
        filter: Document,
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
            .find(filter, find_options)
            .await
            .map_err(|err| {
                warn!(
                    "model::CollectionModel::find_latests could not find latest: {}",
                    err
                );
                err
            }) {
            Ok(cursor) => Some(
                cursor
                    .filter_map(|doc| async { doc.ok() })
                    .collect::<Vec<_>>()
                    .await,
            ),
            Err(err) => {
                warn!(
                    "error fetching in collection {}: {}",
                    self.collection().name(),
                    err
                );
                None
            }
        }
    }
    /// find_with_limits allows to use multiple fields to request documents through the `field_in` parameters.
    /// It also allows to define different limit for the different fields used through the `limits_in` parameter.
    /// For example, I want 10 documents matching the field "channel_id": 1,
    /// and 30 documents matching the field "channel_id": 2, ordered DESC by `created_at`:
    ///
    /// find_with_limits("a_field", vec![1, 2], HashMap::from([(1, 10), (2, 30)]), 10, Some("created_at", SortOrder::DESC))
    async fn find_with_limits<L: Eq + PartialEq<P> + Ord + PartialOrd + Sized + Debug + Display>(
        &self,
        field: &str,
        field_in: Vec<i32>,
        limits_in: impl Into<Option<HashMap<L, i64>>>,
        mut max_limit: i64,
        sort_tuple: impl Into<Option<(&str, SortOrder)>>,
        matches: impl Into<Option<Vec<(&str, Document)>>>,
    ) -> Option<Vec<T>> {
        let mut limits_safe = HashMap::new();
        if let Some(limits_in_into) = limits_in.into() {
            limits_safe = limits_in_into;
            max_limit = limits_safe.iter().map(|e| *e.1).max().unwrap_or(max_limit);
        }
        let mut pipeline = vec![];
        if let Some((field_name, order)) = sort_tuple.into() {
            pipeline.push(doc! { "$sort": {field_name: order.value()} });
        }

        let mut mtch = doc! { "$match": {
            field: {
                "$in": to_bson_vec(&field_in)
            },
        } };

        if let Some(matches_into) = matches.into() {
            if let Ok(match_doc) = mtch.get_document_mut("$match") {
                matches_into.iter().for_each(|item| {
                    match_doc.insert(item.0, item.1.clone());
                });
            }
        }
        let mut rest = vec![
            mtch,
            doc! { "$group": {
                "_id": format!("${}", field),
                "docs": { "$push": "$$ROOT" }
            }},
            doc! { "$project": {
                "_id": 0,
                "link": 1,
                "docs": { "$slice": ["$docs", max_limit] }
            }},
        ];
        pipeline.append(&mut rest);
        let mut cursor = self
            .collection()
            .aggregate(pipeline, None)
            .await
            .map_err(|err| {
                warn!(
                    "model::CollectionModel::find_with_limits could not find latest: {}",
                    err
                );
                err
            })
            .ok()?;
        let mut results = Vec::<T>::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => match mongodb::bson::from_document::<DocsWrapper<T>>(doc) {
                    Ok(t) => {
                        let limit = t.docs.last()
                            .and_then(|t_item| t_item.get_primary_id())
                            .and_then(|p_item| limits_safe.iter().find(|el| el.0 == &p_item).map(|l| *l.1))
                            .unwrap_or(max_limit);
                        t.docs.iter().take(limit as usize).for_each(|inner_doc| results.push(inner_doc.clone()))
                    },
                    Err(e) => warn!("model::CollectionModel::find_with_limits failed to deserialize document: {}", e),
                },
                Err(e) => warn!("model::CollectionModel::find_with_limits failed to retrieve document: {}", e),
            }
        }
        Some(results)
    }
    /// find_latests returns a `limit` amount of documents
    /// ordered by `sort` (SortOrder) on a `field`.
    /// If `field` is an integer, `after` can be used to fetch
    /// documents that are greater than `field`.
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
            None => doc! {},
        };
        let after_into = after.into();
        if after_into.is_some() {
            filter_options.insert(
                field,
                doc! {
                    "$gt": after_into.unwrap(),
                },
            );
        }

        self.collection()
            .find(filter_options, find_options)
            .await
            .map_err(|err| {
                warn!(
                    "model::CollectionModel::find_latests could not find latest: {}",
                    err
                );
                err
            })
            .ok()?
            .try_collect()
            .await
            .map_err(|err| {
                warn!(
                    "model::CollectionModel::find_latests could collect: {}",
                    err
                );
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
    /// get_next_seq requires a "counters" collection to exist, or writing rights to create it.
    /// It will then try to fetch a document, containing a `seq` field, matching the collection's name as its `_id`.
    /// If does not exist, a new document with the previously mentioned specifics will be created, setting `seq` to `0`.
    /// Then, `seq` will be incremented by 1, and the document updated in the collection.
    /// Finally, the updated `seq` will be returned.
    async fn get_next_seq(&self) -> mongodb::error::Result<i32> {
        let counters = self
            .get_diff_collection::<Counter>("counters")
            .ok_or(db_not_found_err())?;
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
            .ok_or_else(|| db_not_found_err());

        result_doc.map(|doc| doc.seq)
    }

    async fn get_seq(&self, id: &str) -> mongodb::error::Result<i32> {
        let counters = self
            .get_diff_collection::<Counter>("counters")
            .ok_or(db_not_found_err())?;
        let res = counters.find_one(doc! {"_id": id}, None).await?;
        match res {
            Some(r) => Ok(r.seq),
            None => self.get_next_seq().await,
        }
    }
}

pub trait FieldSort<V> {
    fn sort_by_value(&self) -> V;
}

pub trait PrimaryID<T: PartialEq> {
    fn get_primary_id(&self) -> Option<T>;
}

pub struct BlankCollection<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    db_name: &'a str,
}

impl<'a, P: PartialEq + Into<mongodb::bson::Bson> + Clone, T: CollectionModelConstraint<P>>
    CollectionModel<P, T> for BlankCollection<'a, T>
{
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

impl<'a, T: CollectionModelConstraint<String>> BlankCollection<'a, T> {
    pub fn new(handle: &'a Handle, db_name: &'a str, collection_name: &str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Error::str_to_result("no database found"),
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

    use super::{BlankCollection, FieldSort, PrimaryID};
    use crate::{
        config,
        db::{
            self,
            items::Items,
            model::{CollectionModel, SortOrder},
        },
        entities::potential_articles::PotentialArticle,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct Test {
        link: String,
    }
    impl FieldSort<String> for Test {
        fn sort_by_value(&self) -> String {
            "pog".to_string()
        }
    }

    impl PrimaryID<String> for Test {
        fn get_primary_id(&self) -> Option<String> {
            None
        }
    }

    #[rocket::async_test]
    async fn test_get_seq() {
        let settings = config::Settings::new().unwrap();
        let db_handle = db::mongo::get_handle(&settings).await;
        let coll = BlankCollection::<Test>::new(&db_handle, "panya", "test").unwrap();
        // coll.insert_many(&[Test{}]).await.unwrap();

        println!("next sequence: {}", coll.get_next_seq().await.unwrap());
    }

    #[rocket::async_test]
    async fn test_find_with_limits() {
        let settings = config::Settings::new().unwrap();
        let db_handle = db::mongo::get_handle(&settings).await;
        let coll = Items::<PotentialArticle>::new(&db_handle, "panya").unwrap();
        // coll.insert_many(&[Test{}]).await.unwrap();

        println!(
            "next find_with_limits: {:?}",
            coll.find_with_limits(
                "channel_id",
                vec![1, 2],
                HashMap::from([(1, 10), (2, 5),]),
                10,
                ("create_date", SortOrder::DESC),
                None,
            )
            .await
            .unwrap()
        );
    }
}
