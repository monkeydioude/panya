use serde::{Deserialize, Serialize};

use crate::{
    db::model::Updatable,
    entities::channel::{Channel, SourceType},
};

#[derive(Deserialize, Serialize)]
pub struct PublicChannel {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
    pub url: String,
    pub source_type: Option<SourceType>,
    #[serde(skip_serializing)]
    refresh_frequency: Option<i32>,
}

impl From<&Channel> for PublicChannel {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel.id,
            name: channel.name.clone(),
            url: channel.url.clone(),
            source_type: channel.source_type.into(),
            refresh_frequency: channel.refresh_frequency.into(),
        }
    }
}

impl PublicChannel {
    pub fn from_channels(channels: Vec<Channel>) -> Vec<Self> {
        channels.iter().map(|c| c.into()).collect()
    }
}

impl Updatable<i32, Channel> for PublicChannel {
    fn update(&self, entity: Channel) -> Channel {
        let mut new = entity.clone();

        if let Some(source_type) = self.source_type {
            new.source_type = source_type;
        }
        if let Some(refresh_frequency) = self.refresh_frequency {
            new.refresh_frequency = refresh_frequency;
        }
        new.name = self.name.clone();
        new
    }
}
