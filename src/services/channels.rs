use crate::{entities::channel::SourceType, error::Error};

pub fn compute_refresh_avg(current_avg: f32, time_to_refresh: i64, refresh_count: i32) -> f32 {
    // multiplying by 1000 to avoid to lose f32's 3 decimals precision
    (((refresh_count as i64 * (current_avg * 1000.) as i64) + (time_to_refresh * 1000))
        / ((refresh_count + 1) as i64 * 1000)) as f32
}

static RSS_CONTENT_TYPES: [&str; 2] = ["application/xml", "application/rss"];

pub async fn find_out_source_type(channel_name: &str) -> Result<SourceType, Error> {
    let response = reqwest::get(channel_name).await?;
    let content_type = match response.headers().get("content-type") {
        Some(header) => header.to_str().unwrap_or_default(),
        None => "",
    };
    if RSS_CONTENT_TYPES
        .iter()
        .any(|ct| content_type.find(ct).is_some())
    {
        return Ok(SourceType::RSSFeed);
    }
    Ok(SourceType::Bakery)
}
