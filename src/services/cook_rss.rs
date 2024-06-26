use std::collections::BTreeMap;
use rss::{ChannelBuilder, Item};
use crate::entities::potential_articles::PotentialArticle;

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

/// cook turns a vec of articles into xml using the std rss::ChannelBuilder
pub fn cook(link: &str, title: &str, articles: Vec<PotentialArticle>) -> String {
    let mut items = vec![];
    for value in articles.iter() {
        items.push(Item {
            title: value.some_desc(),
            link: Some(value.link.to_string()),
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
        });
    }

    ChannelBuilder::default()
        .title(title.to_string())
        .link(link.to_string())
        .description(title.to_string())
        .items(items)
        .build()
        .to_string()
}
