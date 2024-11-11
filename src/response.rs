use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HTTPResponse {
    pub message: String,
    pub code: u8,
}

pub fn answer(message: &str, code: u8) -> HTTPResponse {
    HTTPResponse {
        message: message.to_string(),
        code,
    }
}

impl HTTPResponse {
    pub fn ok() -> HTTPResponse {
        answer("ok", 200)
    }
    pub fn created() -> HTTPResponse {
        answer("created", 201)
    }
}
