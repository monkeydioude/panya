#![feature(async_fn_in_trait)]
#[macro_use]
extern crate rocket;

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod services;
pub mod utils;

use handlers::panya::get_url;
use rocket::{Build, Config, Rocket, Route};
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
            ..Config::default()
        })
        .mount("/panya", routes)
        .manage(db::mongo::get_handle(&settings).await)
        .manage(settings)
}

#[launch]
async fn launch() -> _ {
    lezgong(routes![healthcheck, get_url], 8083).await
}