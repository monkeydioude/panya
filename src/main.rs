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

use handlers::{feed::get_feed, list::get_list, panya::get_url};
use rocket::{fairing::AdHoc, Build, Config, Rocket, Route, info};

use utils::now_timestamp_ms;
use std::net::Ipv4Addr;

#[get("/healthcheck")]
fn healthcheck() -> &'static str {
    "Ok"
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
}

#[launch]
async fn launch() -> _ {
    lezgong(routes![healthcheck, get_url, get_feed, get_list], 8083).await
}
