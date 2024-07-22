#![allow(async_fn_in_trait)]
#[macro_use]
extern crate rocket;

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod services;
pub mod utils;
pub mod converters;
pub mod entities;

use handlers::{channel::{add_url, delete_url, get_channel_list}, feed::get_feed, panya::get_url};
use rocket::{fairing::{AdHoc, Fairing, Info, Kind}, Build, Config, Data, Request, Response, Rocket, Route};

use utils::now_timestamp_ms;
use uuid::Uuid;
use std::net::Ipv4Addr;

const X_REQUEST_ID_LABEL: &str = "X-Request-ID";
const NO_X_REQUEST_ID_LABEL: &str = "no_x_request_id";

#[get("/healthcheck")]
fn healthcheck() -> &'static str {
    "Ok"
}

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
        response.set_raw_header(X_REQUEST_ID_LABEL, req.local_cache(|| "".to_string()));
    }
}

async fn lezgong(routes: Vec<Route>, port: u16) -> Rocket<Build> {
    let settings = config::Settings::new().unwrap();
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
        .attach(AdHoc::on_request("time_before", | req, _ | Box::pin(async move {
            req.local_cache(|| now_timestamp_ms());
        })))
        .attach(AdHoc::on_response("time_after", | req, res | Box::pin(async move {
            let time = req.local_cache(|| 0 as u128);
            info!("request: {:?}\nresponse: {:?}\nstatus: {}\nexec time: {:}", req.uri(), res, res.status(), now_timestamp_ms() - time);
        })))
        .manage(db::mongo::get_handle(&settings).await)
        .manage(settings)
        .attach(XRequestIdMiddleware)
}

#[launch]
async fn launch() -> _ {
    lezgong(routes![healthcheck, get_url, get_feed, get_channel_list, add_url, delete_url], 8083).await
}
