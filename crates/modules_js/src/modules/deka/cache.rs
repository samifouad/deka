use std::collections::HashMap;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use deno_core::{error::CoreError, op2};

use super::fs::resolve_path;

struct CachedSource {
    modified: Option<std::time::SystemTime>,
    source: String,
}

static SOURCE_CACHE: OnceLock<Mutex<HashMap<std::path::PathBuf, CachedSource>>> = OnceLock::new();
static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static CACHE_INVALIDATIONS: AtomicU64 = AtomicU64::new(0);

pub fn module_cache_stats() -> serde_json::Value {
    let entries = SOURCE_CACHE
        .get()
        .and_then(|cache| cache.lock().ok().map(|guard| guard.len()))
        .unwrap_or(0);

    serde_json::json!({
        "entries": entries,
        "hits": CACHE_HITS.load(Ordering::Relaxed),
        "misses": CACHE_MISSES.load(Ordering::Relaxed),
        "invalidations": CACHE_INVALIDATIONS.load(Ordering::Relaxed),
    })
}

#[op2]
#[string]
pub(crate) fn op_read_handler_source(#[string] path: String) -> Result<String, CoreError> {
    let path = resolve_path(&path)?;
    let path = std::fs::canonicalize(&path).unwrap_or(path);
    Ok(read_cached_source(&path)?.source)
}

#[derive(serde::Serialize)]
struct ModuleSourceInfo {
    path: String,
    source: String,
    modified_ms: Option<u64>,
}

#[op2]
#[serde]
pub(crate) fn op_read_module_source(#[string] path: String) -> Result<ModuleSourceInfo, CoreError> {
    let path = resolve_path(&path)?;
    let path = std::fs::canonicalize(&path).unwrap_or(path);
    if let Some(ext) = path.extension().and_then(|value| value.to_str()) {
        if ext == "node" || ext == "wasm" {
            let metadata = std::fs::metadata(&path).map_err(CoreError::from)?;
            let modified = metadata.modified().ok();
            return Ok(ModuleSourceInfo {
                path: path.display().to_string(),
                source: String::new(),
                modified_ms: modified.map(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                        .as_millis() as u64
                }),
            });
        }
    }
    let cached = read_cached_source(&path)?;
    Ok(ModuleSourceInfo {
        path: path.display().to_string(),
        source: cached.source,
        modified_ms: cached.modified.map(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64
        }),
    })
}

fn read_cached_source(path: &std::path::Path) -> Result<CachedSource, CoreError> {
    let cache = SOURCE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let metadata = std::fs::metadata(path).map_err(CoreError::from)?;
    let modified = metadata.modified().ok();
    let mut guard = cache.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Source cache locked",
        ))
    })?;
    if let Some(entry) = guard.get(path) {
        let entry_modified = entry.modified;
        if entry_modified == modified {
            CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            return Ok(CachedSource {
                modified,
                source: entry.source.clone(),
            });
        }
        CACHE_INVALIDATIONS.fetch_add(1, Ordering::Relaxed);
    }
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
    let source = std::fs::read_to_string(path).map_err(CoreError::from)?;
    let cached = CachedSource {
        modified,
        source: source.clone(),
    };
    guard.insert(path.to_path_buf(), cached);
    Ok(CachedSource { modified, source })
}
