use anyhow::{Context, Result, bail};
use serde_json::Value;
use urlencoding::encode;

pub fn fetch_npm_metadata(name: &str) -> Result<Value> {
    let encoded = encode(name);
    let url = format!("https://registry.npmjs.org/{encoded}");
    eprintln!("helper    metadata url {}", url);
    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("failed to fetch npm metadata for {name}"))?;
    if !response.status().is_success() {
        bail!("npm metadata request failed: {}", response.status())
    }
    let value = response
        .json::<Value>()
        .context("failed to parse npm metadata")?;
    Ok(value)
}

pub fn resolve_package_version(metadata: &Value, hint: Option<&str>) -> Option<String> {
    if let Some(hint) = hint {
        if metadata.get("versions").and_then(|v| v.get(hint)).is_some() {
            return Some(hint.to_string());
        }
    }

    if let Some(Value::String(latest)) = metadata.get("dist-tags").and_then(|v| v.get("latest")) {
        return Some(latest.clone());
    }

    if let Some(Value::Object(map)) = metadata.get("versions") {
        if let Some(first) = map.keys().next() {
            return Some(first.clone());
        }
    }

    None
}
