//! T4 Client Management
//!
//! HTTP client for T4 object storage with S3-compatible API.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error as IoError, ErrorKind};
use std::sync::{Arc, Mutex};

/// Client configuration from JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T4ClientConfig {
    /// T4 server URL (e.g., "https://t4.deka.gg")
    pub url: Option<String>,
    /// Bucket name
    pub bucket: Option<String>,
    /// Optional JWT token for authentication
    pub token: Option<String>,
}

/// Managed T4 client with metadata
pub struct T4Client {
    pub http_client: Client,
    pub base_url: String,
    pub bucket: String,
    pub token: Option<String>,
}

impl T4Client {
    /// Create URL for S3-compatible endpoint
    pub fn object_url(&self, key: &str) -> String {
        format!("{}/_s3/{}/{}", self.base_url, self.bucket, key)
    }
}

/// Global client registry
pub struct ClientRegistry {
    clients: Mutex<HashMap<u32, Arc<T4Client>>>,
    next_id: Mutex<u32>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Create a new client and return its ID
    pub fn create_client(&self, config: T4ClientConfig) -> Result<u32, IoError> {
        // Get T4 URL from config or environment
        let base_url = config
            .url
            .or_else(|| std::env::var("T4_URL").ok())
            .ok_or_else(|| {
                IoError::new(
                    ErrorKind::NotFound,
                    "Missing T4_URL (e.g., https://t4.deka.gg)",
                )
            })?;

        let bucket = config
            .bucket
            .or_else(|| std::env::var("T4_BUCKET").ok())
            .unwrap_or_else(|| "default".to_string());

        let token = config.token.or_else(|| std::env::var("T4_TOKEN").ok());

        let http_client = Client::builder().build().map_err(|e| {
            IoError::new(
                ErrorKind::Other,
                format!("Failed to create HTTP client: {}", e),
            )
        })?;

        let t4_client = Arc::new(T4Client {
            http_client,
            base_url,
            bucket,
            token,
        });

        // Store client and return ID
        let mut next_id = self.next_id.lock().unwrap();
        let client_id = *next_id;
        *next_id += 1;

        let mut clients = self.clients.lock().unwrap();
        clients.insert(client_id, t4_client);

        Ok(client_id)
    }

    /// Get a client by ID
    pub fn get_client(&self, client_id: u32) -> Result<Arc<T4Client>, IoError> {
        let clients = self.clients.lock().unwrap();
        clients.get(&client_id).cloned().ok_or_else(|| {
            IoError::new(
                ErrorKind::NotFound,
                format!("Invalid client ID: {}", client_id),
            )
        })
    }
}

// Global registry instance
lazy_static::lazy_static! {
    pub static ref REGISTRY: ClientRegistry = ClientRegistry::new();
}
