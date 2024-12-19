use std::{sync::Arc, thread::sleep, time};

use crate::{
    db::{model::CollectionModel, mongo::Handle, user::Users},
    entities::user::User,
    error::Error,
};
use futures::{StreamExt, TryFutureExt};
use heyo_rpc_client::rpc::{broker_client::BrokerClient, Message, Subscriber};
use rocket::tokio::spawn;
use serde::Deserialize;
use tonic::transport::{Channel, Uri};
use uuid::Uuid;
const USER_CREATION_EVENT: &'static str = "event.on_user_creation";

#[derive(Debug, Deserialize)]
pub struct IdentityUser {
    pub id: i32,
    pub login: String,
    #[serde(default)]
    pub channel_ids: Vec<i32>,
    pub created_at: String,
}
impl Into<User> for IdentityUser {
    fn into(self) -> User {
        User {
            id: self.id,
            username: self.login,
            channel_ids: self.channel_ids,
        }
    }
}

async fn process_message(msg: &Message, db_handle: &Arc<Handle>) -> Result<bool, Error> {
    let identity_user: IdentityUser = match serde_json::from_str(&msg.data) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!(
                "[ERR ] Could not deserialize this message {:?}: {}",
                msg, err
            );
            return Ok(false);
        }
    };
    let coll = Users::<User>::new(&db_handle, "panya")?;
    coll.insert_one(&identity_user.into(), None)
        .map_ok(|_| true)
        .await
}

async fn run_listener(client: &mut BrokerClient<Channel>, db_handle: &Arc<Handle>) {
    let sub_res = client
        .subscription(Subscriber {
            event: USER_CREATION_EVENT.to_owned(),
            client_id: Uuid::new_v4().to_string(),
        })
        .await;
    let mut stream = match sub_res {
        Ok(res) => res.into_inner(),
        Err(err) => {
            eprintln!(
                "[ERR ] Could not read setup subscription to the QUEUE: {:?}",
                err
            );
            return;
        }
    };
    while let Some(item) = stream.next().await {
        let msg = match item {
            Ok(opt) => opt,
            Err(err) => {
                eprintln!(
                    "[ERR ] Could not read MESSAGE subscription from QUEUE: {:?}",
                    err
                );
                break;
            }
        };
        println!("[INFO] Received from QUEUE: {:?}", &msg);
        match process_message(&msg, db_handle).await {
            Ok(false) => eprintln!("[ERR ] User creation aborted"),
            Ok(true) => println!("[INFO] User created"),
            Err(err) => {
                eprintln!(
                "[ERR ] Could not process MESSAGE subscription from QUEUE: {:?}, EVENT: {}, CLIENT: {}, MESSAGE: {}",
                err, msg.event, msg.client_id, msg.message_id,
            )
            }
        };
    }
}
pub async fn identity_new_user(db_handle: Arc<Handle>) -> Result<(), Error> {
    println!("[INFO] Starting Identity WORKER setup");

    let arc_db_handle = Arc::clone(&db_handle);
    let addr = std::env::var("BROKER_ADDR").unwrap_or_else(|_| "http://[::]:8022".to_string());
    // Parse `addr` into a `tonic::transport::Uri`
    let uri: Uri = addr
        .parse()
        .expect("Failed to parse BROKER_ADDR into a valid Uri");

    let _ = spawn(async move {
        loop {
            let mut client = match BrokerClient::connect(uri.clone()).await {
                Ok(cl) => cl,
                Err(err) => {
                    println!(
                        "[WARN] Could not connect to the QUEUE: BrokerClient: {}",
                        err
                    );
                    println!("[WARN] QUEUE CLOSED. Retry connecting in 5s...");
                    sleep(time::Duration::from_secs(5));
                    continue;
                }
            };
            println!("[INFO] Identity WORKER is CONNECTED to the QUEUE");
            run_listener(&mut client, &arc_db_handle).await;
            println!("[WARN] Queue CLOSED. Retry connecting in 5s...");
            sleep(time::Duration::from_secs(5));
        }
    });
    Ok(())
}
