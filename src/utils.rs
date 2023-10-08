use chrono::{Utc, Duration};

pub fn now_minus_minutes(minus_minutes: i64) -> i64 {
    (Utc::now() - Duration::minutes(minus_minutes)).timestamp()
}