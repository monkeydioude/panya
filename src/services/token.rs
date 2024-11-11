use crate::error::Error;

#[derive(Debug)]
pub enum ApiTokenError {
    Missing,
    Invalid,
    OtherError,
}

impl From<ApiTokenError> for Error {
    fn from(value: ApiTokenError) -> Self {
        Error(
            match value {
                ApiTokenError::Missing => "Missing JWT",
                ApiTokenError::Invalid => "Invalid JWT",
                ApiTokenError::OtherError => "Internal error (JWT)",
            }
            .to_string(),
        )
    }
}
