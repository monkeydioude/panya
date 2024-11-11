use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use url::{Position, Url};

use crate::error::Error;

pub fn now_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn datetime_minus_minutes(minus_minutes: i64, dt: DateTime<Utc>) -> i64 {
    (dt - Duration::minutes(minus_minutes)).timestamp()
}

pub fn now_minus_minutes(minus_minutes: i64) -> i64 {
    datetime_minus_minutes(minus_minutes, Utc::now())
}

pub fn clean_url(input_url: &str) -> Result<String, url::ParseError> {
    let mut url = Url::parse(input_url)?;
    url.set_query(None);
    url.set_fragment(None);
    let _ = url.set_scheme("");
    let mut cleaned_url = url[..Position::AfterPath].to_string();

    if cleaned_url.ends_with('/') {
        cleaned_url.pop();
    }
    let scheme_end = cleaned_url.find("://").map(|i| i + 3).unwrap_or(0);

    let cleaned_url_no_scheme = &cleaned_url[scheme_end..];

    Ok(cleaned_url_no_scheme.to_string())
}

pub fn decode_base64_url<T: for<'a> Deserialize<'a>>(input: &str) -> Result<T, Error> {
    // Base64 character set (standard and URL-safe replacements)
    let base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let url_safe_base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    // Determine if the input is URL-safe and map characters accordingly
    let base64_chars = if input.contains('-') || input.contains('_') {
        url_safe_base64_chars
    } else {
        base64_chars
    };

    // Pad the input if necessary
    let padded_input = input.trim_end_matches('=');
    let mut buffer = Vec::new();

    for chunk in padded_input.as_bytes().chunks(4) {
        let mut chunk_value: u32 = 0;

        for (i, &byte) in chunk.iter().enumerate() {
            let index = base64_chars.find(byte as char).ok_or_else(|| {
                Error::from(format!("Invalid character in input: {}", byte as char))
            })?;
            chunk_value |= (index as u32) << (18 - i * 6);
        }

        for i in (0..3).filter(|&i| i < chunk.len() - 1) {
            let byte = ((chunk_value >> (16 - i * 8)) & 0xFF) as u8;
            buffer.push(byte);
        }
    }

    serde_json::from_slice(&buffer).map_err(|err| Error::from(err))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::request_guards::auth::Claims;

    use super::*;

    #[test]
    fn test_datetime_minus_x_minutes() {
        let tt = 1696769957;
        let mins = 2;

        assert_eq!(
            datetime_minus_minutes(mins, Utc.timestamp_opt(tt, 0).unwrap()),
            tt - (60 * mins),
        );
    }

    #[test]
    fn test_i_can_clean_url() {
        let trial = "https://www3.nhk.or.jp/news/easy/?limit=5";
        let goal = "www3.nhk.or.jp/news/easy";

        assert_eq!(clean_url(trial).unwrap(), goal);
    }

    #[test]
    fn test_i_can_decode_base64_url() {
        let encoded = "eyJleHBpcmUiOjE3MzA0ODYwNTEsInJlZnJlc2giOjE3MzEwODcyNTEsInVpZCI6N30";
        let c = Claims {
            uid: 7,
            expire: 1730486051,
            refresh: 1731087251,
        };
        assert_eq!(c, decode_base64_url(encoded).unwrap());
    }
}
