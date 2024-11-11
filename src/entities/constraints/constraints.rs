use crate::error::Error;

pub trait Constraint<T> {
    fn assert(&self) -> Result<bool, Error>;
}
