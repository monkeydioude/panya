use std::collections::BTreeMap;

use rss::{ChannelBuilder, Item};
use url::Url;

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

fn get_schema(link: &str) -> String {
    match Url::parse(link) {
        Ok(parts) => parts.scheme().to_string() + "://",
        Err(err) => {
            warn!("could not parse url {}: {}", link, err);
            "https://".to_string()
        }
    }
}

pub fn cook(link: &str, title: &str, articles: Vec<PotentialArticle>) -> String {
    let mut items = vec![];
    let schema = get_schema(link);
    for value in articles.iter() {
        items.push(Item {
            title: value.some_desc(),
            link: Some(schema.to_string() + &value.link),
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
