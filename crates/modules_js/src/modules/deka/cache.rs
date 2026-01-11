use std::collections::HashMap;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::path::PathBuf;

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

// VFS storage for compiled binaries
static VFS_ROOT: OnceLock<Mutex<Option<VfsInfo>>> = OnceLock::new();

struct VfsInfo {
    /// Root directory of the VFS (for path normalization)
    root: PathBuf,
    /// VFS file cache (relative path → content)
    files: HashMap<String, String>,
}

/// Mount a VFS for the current process
pub fn mount_vfs(root: PathBuf, files: HashMap<String, String>) {
    let vfs_info = VfsInfo { root, files };
    let vfs = VFS_ROOT.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = vfs.lock() {
        *guard = Some(vfs_info);
    }
}

/// Check if VFS is mounted
pub fn is_vfs_mounted() -> bool {
    VFS_ROOT
        .get()
        .and_then(|vfs| vfs.lock().ok())
        .and_then(|guard| guard.as_ref().map(|_| true))
        .unwrap_or(false)
}

/// Try to read a file from VFS (DNS-style: VFS first, filesystem fallback)
fn try_read_from_vfs(path: &std::path::Path) -> Option<String> {
    let vfs = VFS_ROOT.get()?;
    let guard = vfs.lock().ok()?;
    let vfs_info = guard.as_ref()?;

    let path_str = path.to_string_lossy();

    // Debug logging
    if std::env::var("DEKA_VFS_DEBUG").is_ok() {
        eprintln!("[vfs-debug] Looking for: {}", path_str);
        eprintln!("[vfs-debug] VFS root: {}", vfs_info.root.display());
        eprintln!("[vfs-debug] VFS files: {:?}", vfs_info.files.keys().collect::<Vec<_>>());
    }

    // Try exact match first
    if let Some(content) = vfs_info.files.get(path_str.as_ref()) {
        if std::env::var("DEKA_VFS_DEBUG").is_ok() {
            eprintln!("[vfs-debug] ✓ Found exact match: {}", path_str);
        }
        return Some(content.clone());
    }

    // Try relative to VFS root
    if let Ok(relative) = path.strip_prefix(&vfs_info.root) {
        let relative_str = relative.to_string_lossy().to_string();
        if let Some(content) = vfs_info.files.get(&relative_str) {
            if std::env::var("DEKA_VFS_DEBUG").is_ok() {
                eprintln!("[vfs-debug] ✓ Found relative match: {}", relative_str);
            }
            return Some(content.clone());
        }
    }

    // Try filename only (for files in VFS root)
    if let Some(filename) = path.file_name() {
        let filename_str = filename.to_string_lossy().to_string();
        if let Some(content) = vfs_info.files.get(&filename_str) {
            if std::env::var("DEKA_VFS_DEBUG").is_ok() {
                eprintln!("[vfs-debug] ✓ Found filename match: {}", filename_str);
            }
            return Some(content.clone());
        }
    }

    if std::env::var("DEKA_VFS_DEBUG").is_ok() {
        eprintln!("[vfs-debug] ✗ Not found in VFS");
    }

    None
}

pub fn module_cache_stats() -> serde_json::Value {
    let entries = SOURCE_CACHE
        .get()
        .and_then(|cache| cache.lock().ok().map(|guard| guard.len()))
        .unwrap_or(0);

    let vfs_mounted = is_vfs_mounted();
    let vfs_files = VFS_ROOT
        .get()
        .and_then(|vfs| vfs.lock().ok())
        .and_then(|guard| guard.as_ref().map(|info| info.files.len()))
        .unwrap_or(0);

    serde_json::json!({
        "entries": entries,
        "hits": CACHE_HITS.load(Ordering::Relaxed),
        "misses": CACHE_MISSES.load(Ordering::Relaxed),
        "invalidations": CACHE_INVALIDATIONS.load(Ordering::Relaxed),
        "vfs_mounted": vfs_mounted,
        "vfs_files": vfs_files,
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
    // DNS-style: Check VFS first!
    if let Some(vfs_content) = try_read_from_vfs(path) {
        return Ok(CachedSource {
            modified: None, // VFS files don't have modification times
            source: vfs_content,
        });
    }

    // Fall back to filesystem
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
