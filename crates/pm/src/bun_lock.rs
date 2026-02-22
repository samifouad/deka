use crate::spec::parse_package_spec;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;

#[allow(dead_code)]
#[derive(Clone)]
pub struct BunLockPackage {
    pub name: String,
    pub version: String,
    pub descriptor: String,
    pub metadata: Value,
    pub integrity: Option<String>,
}

pub struct BunLock {
    packages: HashMap<String, BunLockPackage>,
}

impl BunLock {
    pub fn load() -> Result<Option<Self>> {
        let content = match fs::read_to_string("bun.lock") {
            Ok(value) => value,
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(err).context("failed to read bun.lock");
            }
        };

        let json: Value =
            json5::from_str(&content).context("failed to parse bun.lock (json5 compatible)")?;
        let packages = match json.get("packages").and_then(|value| value.as_object()) {
            Some(map) => map,
            None => return Ok(None),
        };

        let mut entries = HashMap::new();
        for (key, value) in packages {
            if let Value::Array(items) = value {
                if let Some(descriptor) = items.get(0).and_then(Value::as_str) {
                    let (name, version_opt) = parse_package_spec(descriptor);
                    if let Some(version) = version_opt {
                        let metadata = items
                            .get(2)
                            .cloned()
                            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
                        let integrity = items
                            .get(3)
                            .and_then(Value::as_str)
                            .map(|value| value.to_string());

                        let entry = BunLockPackage {
                            name: name.clone(),
                            version: version.clone(),
                            descriptor: descriptor.to_string(),
                            metadata,
                            integrity,
                        };
                        entries.entry(name.clone()).or_insert_with(|| entry.clone());
                        if key != &name {
                            entries.entry(key.clone()).or_insert_with(|| entry.clone());
                        }
                    }
                }
            }
        }

        Ok(Some(Self { packages: entries }))
    }

    pub fn lookup(&self, lock_key: Option<&str>, name: &str) -> Option<&BunLockPackage> {
        if let Some(key) = lock_key {
            if let Some(entry) = self.packages.get(key) {
                if std::env::var("DEKA_DEBUG").is_ok() {
                    eprintln!("[DEBUG LOOKUP] Found '{}' in lockfile", key);
                }
                return Some(entry);
            } else {
                if std::env::var("DEKA_DEBUG").is_ok() {
                    eprintln!(
                        "[DEBUG LOOKUP] Key '{}' not found, falling back to '{}'",
                        key, name
                    );
                }
            }
        }
        let result = self.packages.get(name);
        if std::env::var("DEKA_DEBUG").is_ok() {
            if result.is_some() {
                eprintln!("[DEBUG LOOKUP] Found '{}' in lockfile (fallback)", name);
            } else {
                eprintln!("[DEBUG LOOKUP] '{}' not found in lockfile", name);
            }
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&BunLockPackage> {
        self.packages.get(key)
    }
}
