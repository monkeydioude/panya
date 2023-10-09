use std::collections::HashMap;

use mongodb::{options::ClientOptions, Client, Database};

use crate::config::Settings;

#[derive(Debug)]
pub struct Handle {
    client: Client,
    databases: HashMap<String, Database>,
}

impl Handle {
    pub fn client(&self) -> Option<&Client> {
        Some(&self.client)
    }

    pub fn database(&self, db_name: &str) -> Option<&Database> {
        self.databases.get(db_name)
    }

    pub async fn new(settings: &Settings) -> Self {
        let mut client_options = ClientOptions::parse(&settings.db_path).await.unwrap();
        client_options.app_name = Some(settings.app_name.clone());
        let client = Client::with_options(client_options).unwrap();
        let databases = settings
            .databases
            .iter()
            .map(|name| (name.clone(), client.database(name)))
            .collect();

        Handle { client, databases }
    }
}

pub async fn get_handle(settings: &Settings) -> Handle {
    Handle::new(settings).await
}
