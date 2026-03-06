use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphqlResponse {
    pub data: Option<Value>,
    pub errors: Option<Vec<ApiErrorEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorEntry {
    pub code: Option<i64>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub auth_token: String,
    pub ct0: String,
}
