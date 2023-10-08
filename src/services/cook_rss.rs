use std::collections::BTreeMap;

use rss::{ChannelBuilder, Item};

use super::bakery::PotentialArticle;

impl From<PotentialArticle> for Item {
    fn from(value: PotentialArticle) -> Self {
        Item {
            title: value.some_desc(),
            link: value.some_link(),
            description: value.some_desc(),
            author: None,
            categories: vec![],
            comments: None,
            enclosure: None,
            guid: None,
            pub_date: value.some_human_date(),
            source: None,
            content: None,
            extensions: BTreeMap::new(),
            itunes_ext: None,
            dublin_core_ext: None,
        }
    }
}

pub fn cook(articles: Vec<PotentialArticle>) -> String {
    let mut items = vec![];
    for v in articles.iter() {
        items.push(v.clone().into());
    }
    
    ChannelBuilder::default()
        .title("test".to_string())
        .link("test".to_string())
        .description("lezgong".to_string())
        .items(items)
        .build()
        .to_string()
}