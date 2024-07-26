use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ServiceHealth {
    health: String,
}

#[get("/healthcheck")]
pub fn healthcheck() -> Json<ServiceHealth> {
    Json(ServiceHealth {
        health: "OK".to_string(),
    })
}
