use crate::db::{
    entities::Refresh,
    model::BlankCollection,
};

pub type Refresher<'a> = BlankCollection<'a, Refresh>;

impl<'a> Refresher<'a> {
}
