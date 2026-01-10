use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServeMode {
    Js,
    Static,
    Php,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ServeConfig {
    pub mode: Option<ServeMode>,
    pub entry: Option<String>,
    pub directory_listing: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedHandler {
    pub path: PathBuf,
    pub directory: PathBuf,
    pub mode: ServeMode,
    pub config: ServeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticServeConfig {
    #[serde(default)]
    pub public: Option<String>,

    #[serde(default = "default_clean_urls")]
    pub clean_urls: CleanUrls,

    #[serde(default)]
    pub rewrites: Vec<Rewrite>,

    #[serde(default)]
    pub redirects: Vec<Redirect>,

    #[serde(default)]
    pub headers: Vec<Header>,

    #[serde(default = "default_directory_listing")]
    pub directory_listing: DirectoryListing,

    #[serde(default = "default_unlisted")]
    pub unlisted: Vec<String>,

    #[serde(default)]
    pub trailing_slash: Option<bool>,

    #[serde(default)]
    pub render_single: bool,

    #[serde(default)]
    pub symlinks: bool,

    #[serde(default)]
    pub server_routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CleanUrls {
    Bool(bool),
    Patterns(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DirectoryListing {
    Bool(bool),
    Patterns(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewrite {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Redirect {
    pub source: String,
    pub destination: String,
    #[serde(default = "default_redirect_type")]
    pub r#type: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub source: String,
    pub headers: Vec<HeaderEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderEntry {
    pub key: String,
    pub value: Option<String>,
}

impl ServeConfig {
    pub fn load(directory: &Path) -> Self {
        let config_path = directory.join("serve.json");
        if !config_path.exists() {
            return Self::default();
        }

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(_) => return Self::default(),
        };

        serde_json::from_str::<ServeConfig>(&contents).unwrap_or_default()
    }
}

impl Default for StaticServeConfig {
    fn default() -> Self {
        Self {
            public: None,
            clean_urls: default_clean_urls(),
            rewrites: Vec::new(),
            redirects: Vec::new(),
            headers: Vec::new(),
            directory_listing: default_directory_listing(),
            unlisted: default_unlisted(),
            trailing_slash: None,
            render_single: false,
            symlinks: false,
            server_routes: Vec::new(),
        }
    }
}

impl StaticServeConfig {
    pub fn load(directory: &Path) -> Self {
        let config_path = directory.join("serve.json");
        if !config_path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str::<StaticServeConfig>(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

pub fn resolve_handler_path(path: &str) -> Result<ResolvedHandler, String> {
    let path = Path::new(path);
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cwd = std::env::current_dir().map_err(|e| format!("Failed to get cwd: {}", e))?;
        cwd.join(path)
    };

    let abs_path = if abs_path.exists() {
        abs_path.canonicalize().unwrap_or(abs_path)
    } else {
        abs_path
    };

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

    if let Some(ref entry) = serve_config.entry {
        let entry_path = if Path::new(entry).is_absolute() {
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
            directory: handler_dir,
            mode,
            config: serve_config,
        });
    }

    if !is_dir {
        let mode = serve_config
            .mode
            .clone()
            .unwrap_or_else(|| detect_mode(&abs_path));
        return Ok(ResolvedHandler {
            path: abs_path,
            directory: handler_dir,
            mode,
            config: serve_config,
        });
    }

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
                            directory: handler_dir,
                            mode,
                            config: serve_config,
                        });
                    }
                }
            }
        }
    }

    let index_files = [
        "index.php",
        "index.html",
        "index.js",
        "index.ts",
        "main.ts",
        "main.js",
        "handler.js",
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
                directory: handler_dir,
                mode,
                config: serve_config,
            });
        }
    }

    Ok(ResolvedHandler {
        path: abs_path,
        directory: handler_dir,
        mode: serve_config.mode.clone().unwrap_or(ServeMode::Static),
        config: serve_config,
    })
}

fn detect_mode(path: &Path) -> ServeMode {
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

fn default_clean_urls() -> CleanUrls {
    CleanUrls::Bool(true)
}

fn default_directory_listing() -> DirectoryListing {
    DirectoryListing::Bool(true)
}

fn default_unlisted() -> Vec<String> {
    vec![".DS_Store".to_string(), ".git".to_string()]
}

fn default_redirect_type() -> u16 {
    301
}
