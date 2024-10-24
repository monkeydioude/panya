use super::{
    model::{CollectionModel, CollectionModelConstraint},
    mongo::Handle,
};
use crate::{entities::channel::Channel, error::Error};
use chrono::Utc;
use mongodb::{bson::doc, results::InsertManyResult, Collection, Database};
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Channels<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    db_name: String,
}

impl<'a> Channels<'a, Channel> {
    pub async fn insert_many(
        &self,
        data: &[Channel],
        _: Option<String>,
    ) -> Result<InsertManyResult, Error> {
        // let idx = index.unwrap_or_else(|| "create_date".to_string());
        // // Works cause we dont store result, nor do we return it.
        // // An Err() is returned, if that's the case.
        // self.collection()
        //     // Oftenly creating new collectionm therefore index
        //     .create_index(IndexModel::builder().keys(doc! {idx: -1}).build(), None)
        //     .await?;
        CollectionModel::<i32, Channel>::insert_many(self, data).await
    }

    pub fn get_database_name(&self) -> &String {
        &self.db_name
    }

    pub async fn update_refresh(
        &self,
        channel_id: impl Into<Option<i32>>,
        channel_name: impl Into<Option<&str>>,
    ) -> Option<()> {
        let mut doc = None;
        if let Some(id) = channel_id.into() {
            doc = Some(doc! {"id": id});
        } else if let Some(name) = channel_name.into() {
            doc = Some(doc! {"name": name});
        }
        let uw_doc = doc?;

        self.collection()
            .update_one(
                uw_doc,
                doc! {"$set": {
                    "last_refresh": Utc::now().timestamp(),
                }},
                None,
            )
            .await
            .ok()
            .and(Some(()))
    }

    pub fn new(handle: &'a Handle, db_name: &'a str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Err(Error("no database found".to_string())),
        })
        .collection::<Channel>("channels");
        Ok(Channels {
            db_name: db_name.to_string(),
            handle,
            collection,
        })
    }
}

impl<'a, P: PartialEq + Into<mongodb::bson::Bson> + Clone, T: CollectionModelConstraint<P>>
    CollectionModel<P, T> for Channels<'a, T>
{
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
