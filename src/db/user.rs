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
