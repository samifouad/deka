use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct RuntimeConfig {
    pub code_cache: Option<CodeCacheConfig>,
    pub introspect: Option<IntrospectConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServeMode {
    Js,
    Static,
    Php,
}

#[derive(Debug, Default, Deserialize)]
pub struct ServeConfig {
    pub mode: Option<ServeMode>,
    pub entry: Option<String>,
    pub directory_listing: Option<bool>,
}

impl ServeConfig {
    pub fn load(directory: &std::path::Path) -> Self {
        let config_path = directory.join("serve.json");
        if !config_path.exists() {
            return Self::default();
        }

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(err) => {
                tracing::warn!("Failed to read {}: {}", config_path.display(), err);
                return Self::default();
            }
        };

        match serde_json::from_str::<ServeConfig>(&contents) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!("Failed to parse {}: {}", config_path.display(), err);
                Self::default()
            }
        }
    }
}

pub struct ResolvedHandler {
    pub path: PathBuf,
    pub mode: ServeMode,
    pub config: ServeConfig,
}

/// Resolve a handler path, detecting directories and index files
pub fn resolve_handler_path(path: &str) -> Result<ResolvedHandler, String> {
    let path = std::path::Path::new(path);
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cwd = std::env::current_dir().map_err(|e| format!("Failed to get cwd: {}", e))?;
        cwd.join(path)
    };

    // Canonicalize if it exists
    let abs_path = if abs_path.exists() {
        abs_path.canonicalize().unwrap_or(abs_path)
    } else {
        abs_path
    };

    // Check if it's a directory
    let is_dir = abs_path.is_dir();

    let (handler_dir, serve_config) = if is_dir {
        let config = ServeConfig::load(&abs_path);
        (abs_path.clone(), config)
    } else if let Some(parent) = abs_path.parent() {
        let config = ServeConfig::load(parent);
        (parent.to_path_buf(), config)
    } else {
        (PathBuf::from("."), ServeConfig::default())
    };

    // If config has an entry override, use that
    if let Some(ref entry) = serve_config.entry {
        let entry_path = if std::path::Path::new(entry).is_absolute() {
            PathBuf::from(entry)
        } else {
            handler_dir.join(entry)
        };

        if !entry_path.exists() {
            return Err(format!("Entry file not found: {}", entry_path.display()));
        }

        let mode = serve_config
            .mode
            .clone()
            .unwrap_or_else(|| detect_mode(&entry_path));
        return Ok(ResolvedHandler {
            path: entry_path,
            mode,
            config: serve_config,
        });
    }

    // If config has a mode override and we have a specific file, use that
    if !is_dir {
        let mode = serve_config
            .mode
            .clone()
            .unwrap_or_else(|| detect_mode(&abs_path));
        return Ok(ResolvedHandler {
            path: abs_path,
            mode,
            config: serve_config,
        });
    }

    // Check for package.json and use "main" field
    let package_json_path = abs_path.join("package.json");
    if package_json_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&package_json_path) {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(main) = package_json.get("main").and_then(|v| v.as_str()) {
                    let main_path = abs_path.join(main);
                    if main_path.exists() {
                        let mode = serve_config
                            .mode
                            .clone()
                            .unwrap_or_else(|| detect_mode(&main_path));
                        return Ok(ResolvedHandler {
                            path: main_path,
                            mode,
                            config: serve_config,
                        });
                    }
                }
            }
        }
    }

    // Directory: search for index files in priority order
    let index_files = [
        "index.php",
        "index.html",
        "index.js",
        "index.ts",
        "main.ts",
        "main.js",
        "handler.js", // Backward compatibility
        "handler.ts",
    ];

    for index_file in &index_files {
        let index_path = abs_path.join(index_file);
        if index_path.exists() {
            let mode = serve_config
                .mode
                .clone()
                .unwrap_or_else(|| detect_mode(&index_path));
            return Ok(ResolvedHandler {
                path: index_path,
                mode,
                config: serve_config,
            });
        }
    }

    // No index file found - use static directory serving
    Ok(ResolvedHandler {
        path: abs_path,
        mode: serve_config.mode.clone().unwrap_or(ServeMode::Static),
        config: serve_config,
    })
}

fn detect_mode(path: &std::path::Path) -> ServeMode {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "php" => ServeMode::Php,
            "js" | "ts" | "mjs" | "mts" => ServeMode::Js,
            "html" | "htm" => ServeMode::Static,
            _ => ServeMode::Static,
        }
    } else {
        ServeMode::Static
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct CodeCacheConfig {
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct IntrospectConfig {
    pub retention_days: Option<u64>,
    pub db_path: Option<String>,
    pub profiling: Option<bool>,
}

impl RuntimeConfig {
    pub fn load() -> Self {
        let path = match Self::find_config_path() {
            Some(path) => path,
            None => return Self::default(),
        };

        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(_) => return Self::default(),
        };

        match toml::from_str::<RuntimeConfig>(&contents) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!("Failed to parse {}: {}", path.display(), err);
                Self::default()
            }
        }
    }

    pub fn code_cache_enabled(&self) -> Option<bool> {
        self.code_cache.as_ref()?.enabled
    }

    pub fn introspect_retention_days(&self) -> u64 {
        self.introspect
            .as_ref()
            .and_then(|config| config.retention_days)
            .unwrap_or(7)
    }

    pub fn introspect_db_path(&self) -> Option<PathBuf> {
        if let Some(path) = self
            .introspect
            .as_ref()
            .and_then(|config| config.db_path.as_ref())
        {
            return Some(expand_home_path(path));
        }

        default_introspect_db_path()
    }

    pub fn introspect_profiling_enabled(&self) -> bool {
        self.introspect
            .as_ref()
            .and_then(|config| config.profiling)
            .unwrap_or(true)
    }

    fn find_config_path() -> Option<PathBuf> {
        let mut candidates = Vec::new();

        if let Ok(path) = std::env::var("DEKA_RUNTIME_CONFIG") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
            tracing::warn!(
                "DEKA_RUNTIME_CONFIG set but file not found: {}",
                path.display()
            );
            candidates.push(path);
        }

        candidates.push(PathBuf::from("config.toml"));
        candidates.push(PathBuf::from("runtime.toml"));
        candidates.push(PathBuf::from("deka-runtime.toml"));

        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        {
            candidates.push(path.join("deka").join("config.toml"));
            candidates.push(path.join("deka").join("runtime.toml"));
            candidates.push(path.join("deka").join("deka-runtime.toml"));
        }

        candidates.push(PathBuf::from("/etc/deka/config.toml"));
        candidates.push(PathBuf::from("/etc/deka/runtime.toml"));
        candidates.push(PathBuf::from("/etc/deka/deka-runtime.toml"));

        candidates.into_iter().find(|path| path.exists())
    }
}

fn default_introspect_db_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".deka").join("introspect.db"))
}

fn expand_home_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(path)
}
