use chrono::{DateTime, Duration, Utc};

pub fn datetime_minus_minutes(minus_minutes: i64, dt: DateTime<Utc>) -> i64 {
    (dt - Duration::minutes(minus_minutes)).timestamp()
}

pub fn now_minus_minutes(minus_minutes: i64) -> i64 {
    datetime_minus_minutes(minus_minutes, Utc::now())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

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
}
