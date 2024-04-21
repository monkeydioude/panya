use std::collections::BTreeMap;

pub async fn request_rss(
  urls: &Vec<String>,
  _global_item_per_feed: i64,
  _item_per_feed: &Option<BTreeMap<String, i32>>,
) {
  let futures: Vec<_> = urls
    .into_iter()
    .map(|url| {
      reqwest::get(url.to_string())
    })
    .collect();

  let results = futures::future::join_all(futures).await;
  for result in results {
    println!("Result: {}", result.unwrap().url());
  }
}

#[cfg(test)]
mod tests {
    use super::request_rss;

  #[tokio::test]
  async fn test_i_can_get_rss() {
    request_rss(
      &(&["https://4thehoard.com/panya?url=https://www3.nhk.or.jp/news/easy/&limit=10", "https://www.lemonde.fr/economie/rss_full.xml", "https://techcrunch.com/feed/"])
        .iter().map(|s| s.to_string()).collect(), 
        2, 
      &None).await;
  }
}