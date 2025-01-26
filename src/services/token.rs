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

pub fn extract_auth(metadata: &str) -> String {
    metadata
        .split(';') // Split by semicolon
        .find(|part| part.trim_start().starts_with("Authorization=")) // Find the "Authorization=" part
        .and_then(|part| part.trim_start().strip_prefix("Authorization=")) // Remove "Authorization="
        .map(|value| value.trim_matches('"')) // Remove surrounding quotes
        .unwrap_or_default() // Default to empty string if not found
        .to_string()
}
