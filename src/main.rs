#![allow(async_fn_in_trait)]
#[macro_use]
extern crate rocket;

pub mod config;
pub mod converters;
pub mod db;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod request_guards;
pub mod response;
pub mod services;
pub mod utils;
pub mod workers;

use handlers::{
    channel::{add_url, delete_channel, get_channel, get_channel_list, update_channel},
    healthcheck::healthcheck,
    panya::get_url,
    user::{add_user, login_user, show_user, show_user_channels, show_user_feed},
};
use rocket::{
    fairing::{AdHoc, Fairing, Info, Kind},
    Build, Config, Data, Request, Response, Rocket, Route,
};
// use workers::identity::identity_new_user;

use std::{net::Ipv4Addr, sync::Arc};
use utils::now_timestamp_ms;
use uuid::Uuid;

const X_REQUEST_ID_LABEL: &str = "X-Request-ID";
const NO_X_REQUEST_ID_LABEL: &str = "no_x_request_id";

struct XRequestIdMiddleware;

#[rocket::async_trait]
impl Fairing for XRequestIdMiddleware {
    fn info(&self) -> Info {
        Info {
            name: "Uuid handler",
            kind: Kind::Response | Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        let mut uuid = match req.headers().get(X_REQUEST_ID_LABEL).next() {
            Some(value) => value.to_string(),
            None => Uuid::new_v4().to_string(),
        };
        if uuid == "" {
            uuid = NO_X_REQUEST_ID_LABEL.to_string();
        }
        req.local_cache(|| uuid);
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, response: &mut Response<'r>) {
        let uuid = req.local_cache(|| "".to_string());
        response.set_raw_header(X_REQUEST_ID_LABEL, uuid);
    }
}

async fn lezgong(routes: Vec<Route>, port: u16) -> Rocket<Build> {
    let settings = config::Settings::new().unwrap();
    let db_handle = Arc::new(db::mongo::get_handle(&settings).await);
    // let _ = identity_new_user(Arc::clone(&db_handle)).await;
    rocket::build()
        .configure(Config {
            port,
            address: "0.0.0.0"
                .parse::<Ipv4Addr>()
                .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
                .into(),
            log_level: rocket::config::LogLevel::Normal,
            ..Config::default()
        })
        .mount("/panya", routes)
        .manage(db_handle)
        .manage(settings)
        .attach(XRequestIdMiddleware)
        .attach(AdHoc::on_request("time_before", |req, _| {
            Box::pin(async move {
                let cache = req.local_cache(|| "".to_string());
                req.local_cache(|| (cache.clone(), now_timestamp_ms()));
            })
        }))
        .attach(AdHoc::on_response("time_after", |req, res| {
            Box::pin(async move {
                let cache = req.local_cache(|| ("".to_string(), 0 as u128));
                let time = now_timestamp_ms() - cache.1;
                info!(
                    "({}): {} {} in {}.{}ms",
                    cache.0,
                    req.uri().path(),
                    res.status().code,
                    time / 1_000,
                    time,
                );
            })
        }))
}

#[launch]
async fn launch() -> _ {
    // let mut client = AuthClient::connect("http://[::]:9100").await.unwrap();
    // let request = tonic::Request::new(UserRequest {
    //     login: "test@test.com".to_string(),
    //     password: "test".to_string(),
    // });

    // let response = client.login(request).await.unwrap();
    // println!("{:?}", response);
    lezgong(
        routes![
            healthcheck,
            get_url,
            // get_feed,
            get_channel_list,
            get_channel,
            update_channel,
            add_url,
            delete_channel,
            show_user,
            show_user_feed,
            show_user_channels,
            add_user,
            login_user,
        ],
        8083,
    )
    .await
}
