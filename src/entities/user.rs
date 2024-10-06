use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
}
