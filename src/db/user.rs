use crate::error::Error;

use super::{
    model::{CollectionModel, CollectionModelConstraint},
    mongo::Handle,
};
use mongodb::{Collection, Database};
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Users<'a, T: Serialize> {
    collection: Collection<T>,
    handle: &'a Handle,
    db_name: String,
}
impl<'a, T: CollectionModelConstraint<i32>> Users<'a, T> {
    pub fn new(handle: &'a Handle, db_name: &'a str) -> Result<Self, Error> {
        let collection = (match handle.database(db_name) {
            Some(res) => res,
            None => return Err(Error("no database found".to_string())),
        })
        .collection::<T>("users");
        Ok(Users {
            db_name: db_name.to_string(),
            handle,
            collection,
        })
    }
    // pub async fn find_by_id(&self, user: User) -> Option<T> {
    //     self.find_one("id", user.id).await
    // }
}
impl<'a, P: PartialEq + Into<mongodb::bson::Bson> + Clone, T: CollectionModelConstraint<P>>
    CollectionModel<P, T> for Users<'a, T>
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
