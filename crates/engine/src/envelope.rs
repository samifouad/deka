use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(default)]
    pub body_base64: Option<String>,
    #[serde(default)]
    pub upgrade: Option<serde_json::Value>,
}

impl ResponseEnvelope {
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
