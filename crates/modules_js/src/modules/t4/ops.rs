//! T4 HTTP Operations

use super::client::{REGISTRY, T4ClientConfig};
use deno_core::{error::CoreError, op2};
use std::io::{Error as IoError, ErrorKind};

#[op2(async)]
#[smi]
pub async fn op_t4_create_client(#[serde] config: T4ClientConfig) -> Result<u32, CoreError> {
    Ok(REGISTRY.create_client(config)?)
}

#[op2(async)]
#[string]
pub async fn op_t4_get_text(
    #[smi] client_id: u32,
    #[string] key: String,
) -> Result<String, CoreError> {
    let client = REGISTRY.get_client(client_id)?;
    let url = client.object_url(&key);

    let mut req = client.http_client.get(&url);
    if let Some(token) = &client.token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Request failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(IoError::new(
            ErrorKind::Other,
            format!("GET {} failed: {}", url, resp.status()),
        )
        .into());
    }

    resp.text().await.map_err(|e| {
        IoError::new(ErrorKind::Other, format!("Failed to read response: {}", e)).into()
    })
}

#[op2(async)]
#[buffer]
pub async fn op_t4_get_buffer(
    #[smi] client_id: u32,
    #[string] key: String,
) -> Result<Vec<u8>, CoreError> {
    let client = REGISTRY.get_client(client_id)?;
    let url = client.object_url(&key);

    let mut req = client.http_client.get(&url);
    if let Some(token) = &client.token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Request failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(IoError::new(
            ErrorKind::Other,
            format!("GET {} failed: {}", url, resp.status()),
        )
        .into());
    }

    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| {
        IoError::new(ErrorKind::Other, format!("Failed to read response: {}", e)).into()
    })
}

#[op2(async)]
pub async fn op_t4_put(
    #[smi] client_id: u32,
    #[string] key: String,
    #[buffer(copy)] data: Vec<u8>,
    #[string] content_type: String,
) -> Result<(), CoreError> {
    let client = REGISTRY.get_client(client_id)?;
    let url = client.object_url(&key);

    let mut req = client
        .http_client
        .put(&url)
        .header("content-type", content_type)
        .body(data);

    if let Some(token) = &client.token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Request failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(IoError::new(
            ErrorKind::Other,
            format!("PUT {} failed: {}", url, resp.status()),
        )
        .into());
    }

    Ok(())
}

#[op2(async)]
pub async fn op_t4_delete(#[smi] client_id: u32, #[string] key: String) -> Result<(), CoreError> {
    let client = REGISTRY.get_client(client_id)?;
    let url = client.object_url(&key);

    let mut req = client.http_client.delete(&url);
    if let Some(token) = &client.token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Request failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(IoError::new(
            ErrorKind::Other,
            format!("DELETE {} failed: {}", url, resp.status()),
        )
        .into());
    }

    Ok(())
}

#[op2(async)]
pub async fn op_t4_exists(#[smi] client_id: u32, #[string] key: String) -> Result<bool, CoreError> {
    let client = REGISTRY.get_client(client_id)?;
    let url = client.object_url(&key);

    let mut req = client.http_client.head(&url);
    if let Some(token) = &client.token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Request failed: {}", e)))?;
    Ok(resp.status().is_success())
}

#[op2(async)]
#[serde]
pub async fn op_t4_stat(
    #[smi] client_id: u32,
    #[string] key: String,
) -> Result<serde_json::Value, CoreError> {
    let client = REGISTRY.get_client(client_id)?;
    let url = client.object_url(&key);

    let mut req = client.http_client.head(&url);
    if let Some(token) = &client.token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Request failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(IoError::new(
            ErrorKind::Other,
            format!("HEAD {} failed: {}", url, resp.status()),
        )
        .into());
    }

    let headers = resp.headers();

    Ok(serde_json::json!({
        "size": headers.get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        "etag": headers.get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "lastModified": headers.get("last-modified")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "contentType": headers.get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream"),
    }))
}
