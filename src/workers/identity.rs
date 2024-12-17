use std::{sync::Arc, thread::sleep, time};

use crate::{
    db::{mongo::Handle, user::Users},
    entities::user::User,
};
use futures::StreamExt;
use heyo_rpc_client::rpc::{broker_client::BrokerClient, Message, Subscriber};
use rocket::tokio::spawn;
use serde::Deserialize;
use std::error::Error;
use tonic::transport::Channel;
use uuid::Uuid;
const USER_CREATION_EVENT: &'static str = "event.on_user_creation";

async fn process_message(msg: Message, db_handle: &Arc<Handle>) -> Result<(), Box<dyn Error>> {
    let user: Result<User, serde_json::Error> = serde_json::from_str(&msg.data);
    // Users::new(&db_handle, "users");
    Ok(())
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
        if let Err(err) = process_message(msg, db_handle).await {
            eprintln!(
                "[ERR ] Could not process MESSAGE subscription from QUEUE: {:?}, EVENT: {}, CLIENT: {}, MESSAGE: {}",
                err, msg.event, msg.client_id, msg.message_id,
            );
        }
    }
}

pub async fn identity_new_user(db_handle: Arc<Handle>) -> Result<(), Box<dyn Error>> {
    println!("[INFO] Starting Identity WORKER setup");

    let arc_db_handle = Arc::clone(&db_handle);
    let _ = spawn(async move {
        loop {
            let mut client = match BrokerClient::connect("http://[::]:8022").await {
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
