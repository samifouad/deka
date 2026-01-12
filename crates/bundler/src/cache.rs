use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Cached module data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModule {
    /// File path that was cached
    pub path: PathBuf,
    /// Source code content
    pub source: String,
    /// File modification time (for invalidation)
    pub mtime: SystemTime,
    /// Hash of the file content
    pub content_hash: String,
    /// Serialized transformed module (we'll use JSON for now, could use bincode)
    pub transformed_code: String,
    /// List of dependencies (import paths)
    pub dependencies: Vec<String>,
}

/// Module cache that persists to disk
pub struct ModuleCache {
    /// Cache directory (e.g., ~/.config/deka/bundler/cache/)
    cache_dir: PathBuf,
    /// In-memory cache for fast lookups
    memory: HashMap<PathBuf, CachedModule>,
    /// Whether cache is enabled
    enabled: bool,
}

impl ModuleCache {
    /// Create a new module cache
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let enabled = std::env::var("DEKA_BUNDLER_NO_CACHE")
            .map(|v| v != "1" && v != "true")
            .unwrap_or(true);

        let cache_dir = cache_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".config")
                .join("deka")
                .join("bundler")
                .join("cache")
        });

        if enabled {
            // Create cache directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&cache_dir) {
                eprintln!(" [cache] Warning: Failed to create cache directory: {}", e);
            } else {
                eprintln!(" [cache] Initialized at {}", cache_dir.display());
            }
        }

        Self {
            cache_dir,
            memory: HashMap::new(),
            enabled,
        }
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get a cached module if it exists and is valid
    pub fn get(&mut self, path: &Path) -> Option<CachedModule> {
        if !self.enabled {
            return None;
        }

        // Check in-memory cache first
        if let Some(cached) = self.memory.get(path) {
            if self.is_valid(path, cached) {
                return Some(cached.clone());
            } else {
                // Invalid - remove from memory
                self.memory.remove(path);
            }
        }

        // Check disk cache
        let cache_file = self.cache_path(path);
        if cache_file.exists() {
            match self.load_from_disk(&cache_file) {
                Ok(cached) => {
                    if self.is_valid(path, &cached) {
                        // Store in memory for future lookups
                        self.memory.insert(path.to_path_buf(), cached.clone());
                        return Some(cached);
                    } else {
                        // Invalid - delete from disk
                        let _ = fs::remove_file(&cache_file);
                    }
                }
                Err(e) => {
                    eprintln!(" [cache] Failed to load {}: {}", cache_file.display(), e);
                    let _ = fs::remove_file(&cache_file);
                }
            }
        }

        None
    }

    /// Store a module in the cache
    pub fn put(&mut self, path: &Path, cached: CachedModule) {
        if !self.enabled {
            return;
        }

        // Store in memory
        self.memory.insert(path.to_path_buf(), cached.clone());

        // Store to disk
        let cache_file = self.cache_path(path);
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match self.save_to_disk(&cache_file, &cached) {
            Ok(_) => {
                // Success - no logging to keep it quiet
            }
            Err(e) => {
                eprintln!(" [cache] Failed to save {}: {}", cache_file.display(), e);
            }
        }
    }

    /// Check if a cached module is still valid
    fn is_valid(&self, path: &Path, cached: &CachedModule) -> bool {
        // Check if file exists
        if !path.exists() {
            return false;
        }

        // Check modification time
        match fs::metadata(path) {
            Ok(metadata) => {
                match metadata.modified() {
                    Ok(mtime) => {
                        // Valid if mtime matches
                        mtime == cached.mtime
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Get the cache file path for a source file
    fn cache_path(&self, path: &Path) -> PathBuf {
        // Create a hash of the full path to use as cache key
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        // Take first 16 chars of hash for shorter filenames
        let short_hash = &hash[..16];

        self.cache_dir.join(format!("{}.json", short_hash))
    }

    /// Load a cached module from disk
    fn load_from_disk(&self, cache_file: &Path) -> Result<CachedModule, String> {
        let json = fs::read_to_string(cache_file)
            .map_err(|e| format!("Failed to read cache file: {}", e))?;

        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse cache file: {}", e))
    }

    /// Save a cached module to disk
    fn save_to_disk(&self, cache_file: &Path, cached: &CachedModule) -> Result<(), String> {
        let json = serde_json::to_string(cached)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;

        fs::write(cache_file, json)
            .map_err(|e| format!("Failed to write cache file: {}", e))
    }

    /// Clear the entire cache
    pub fn clear(&mut self) -> Result<(), String> {
        self.memory.clear();

        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| format!("Failed to clear cache: {}", e))?;
            fs::create_dir_all(&self.cache_dir)
                .map_err(|e| format!("Failed to recreate cache directory: {}", e))?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let disk_count = if self.cache_dir.exists() {
            fs::read_dir(&self.cache_dir)
                .map(|entries| entries.count())
                .unwrap_or(0)
        } else {
            0
        };

        CacheStats {
            memory_count: self.memory.len(),
            disk_count,
            enabled: self.enabled,
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub memory_count: usize,
    pub disk_count: usize,
    pub enabled: bool,
}

/// Helper to compute file content hash
pub fn hash_file_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_cache_basics() {
        let temp_dir = std::env::temp_dir().join("deka-bundler-test");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut cache = ModuleCache::new(Some(temp_dir.clone()));
        cache.enabled = true; // Force enable for testing

        // Create a test file
        let test_file = temp_dir.join("test.js");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&test_file, "console.log('test');").unwrap();

        let mtime = fs::metadata(&test_file).unwrap().modified().unwrap();

        // Cache a module
        let cached = CachedModule {
            path: test_file.clone(),
            source: "console.log('test');".to_string(),
            mtime,
            content_hash: "abc123".to_string(),
            transformed_code: "transformed".to_string(),
            dependencies: vec!["dep1".to_string()],
        };

        cache.put(&test_file, cached.clone());

        // Retrieve from cache
        let retrieved = cache.get(&test_file).expect("Should retrieve from cache");
        assert_eq!(retrieved.source, "console.log('test');");
        assert_eq!(retrieved.dependencies.len(), 1);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cache_invalidation() {
        let temp_dir = std::env::temp_dir().join("deka-bundler-test-invalidation");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut cache = ModuleCache::new(Some(temp_dir.clone()));
        cache.enabled = true;

        // Create a test file
        let test_file = temp_dir.join("test.js");
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&test_file, "console.log('test');").unwrap();

        let old_mtime = fs::metadata(&test_file).unwrap().modified().unwrap();

        // Cache a module
        let cached = CachedModule {
            path: test_file.clone(),
            source: "console.log('test');".to_string(),
            mtime: old_mtime,
            content_hash: "abc123".to_string(),
            transformed_code: "transformed".to_string(),
            dependencies: vec![],
        };

        cache.put(&test_file, cached);

        // Modify the file (change mtime)
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut file = fs::OpenOptions::new().write(true).open(&test_file).unwrap();
        file.write_all(b"console.log('modified');").unwrap();
        drop(file);

        // Cache should be invalid now
        let retrieved = cache.get(&test_file);
        assert!(retrieved.is_none(), "Cache should be invalidated after file modification");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
