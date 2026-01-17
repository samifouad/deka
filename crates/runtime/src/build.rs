use std::sync::Arc;
use std::time::Instant;
use std::path::PathBuf;

use core::Context;
use engine::{RuntimeEngine, config as runtime_config, set_engine};
use pool::{ExecutionMode, HandlerKey, PoolConfig, RequestData};
use stdio as stdio_log;

use crate::env::init_env;
use crate::extensions::extensions_for_build;

pub fn build(context: &Context) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    if let Err(err) = rt.block_on(build_async(context)) {
        stdio_log::error("build", &err);
    }
}

async fn build_async(context: &Context) -> Result<(), String> {
    init_env();
    let start = Instant::now();

    // Check for debug flag and set LOG_LEVEL for stdio
    let debug = context.args.flags.contains_key("--debug")
        || context.args.flags.contains_key("-v")
        || context.args.flags.contains_key("--verbose");

    if debug {
        unsafe {
            std::env::set_var("LOG_LEVEL", "debug");
        }
    }

    // Initialize module cache
    let mut cache = bundler::ModuleCache::new(None);

    // Handle --clear-cache flag
    if context.args.flags.contains_key("--clear-cache") {
        stdio_log::debug("cache", "clearing cache...");
        cache.clear()
            .map_err(|e| format!("Failed to clear cache: {}", e))?;
        stdio_log::log("cache", "cleared successfully");
        return Ok(());
    }

    if cache.is_enabled() {
        let stats = cache.stats();
        stdio_log::debug("cache", &format!("enabled ({} in memory, {} on disk)", stats.memory_count, stats.disk_count));
    }

    let entry = context
        .args
        .positionals
        .get(0)
        .cloned()
        .or_else(|| std::env::var("DEKA_BUILD_ENTRY").ok())
        .ok_or_else(|| "Missing build entry".to_string())?;

    let outdir = context
        .args
        .params
        .get("--outdir")
        .or_else(|| context.args.params.get("-o"))
        .cloned()
        .or_else(|| std::env::var("DEKA_BUILD_OUTDIR").ok())
        .unwrap_or_else(|| "./dist".to_string());

    // Fast path: Call bundler directly instead of going through V8 isolate
    let entry_path = resolve_entry_path(&entry)?;

    // Create output directory
    let outdir_path = PathBuf::from(&outdir);
    std::fs::create_dir_all(&outdir_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Bundle the entry file
    // Parallel bundler is default (set DEKA_PARALLEL_BUNDLER=0 to disable)
    let use_parallel = std::env::var("DEKA_PARALLEL_BUNDLER")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true);

    // Get target from --target param or env var
    // Default: "browser" (bundles node_modules)
    // Option: "server" (marks node_modules as external)
    let target = context
        .args
        .params
        .get("--target")
        .map(|s| s.as_str())
        .unwrap_or("browser");

    // Set env var for bundler to read
    if target == "server" || target == "node" {
        unsafe {
            std::env::set_var("DEKA_EXTERNAL_NODE_MODULES", "1");
        }
    }

    // Set sourcemap flag
    if context.args.flags.contains_key("--sourcemap") {
        unsafe {
            std::env::set_var("DEKA_SOURCEMAP", "1");
        }
    }

    // Set minify flag
    if context.args.flags.contains_key("--minify") {
        unsafe {
            std::env::set_var("DEKA_MINIFY", "1");
        }
    }

    // Bundle the code (parallel by default)
    let (bundle_code, source_map, css_code) = if use_parallel {
        stdio_log::debug("parallel", &format!("bundling with {} workers", num_cpus::get()));
        let root = PathBuf::from(".").canonicalize()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let bundler = bundler::ParallelBundler::new(root);
        let output = bundler.bundle(&entry_path).await?;
        (output.code, output.map, None)  // Parallel bundler doesn't support CSS yet
    } else {
        // Use standard cached bundler
        let bundle = bundler::bundle_browser_assets_cached(&entry_path, &mut cache)
            .map_err(|e| format!("Bundle failed: {}", e))?;
        (bundle.code, None, bundle.css)
    };

    // Generate filenames
    let entry_path_buf = PathBuf::from(&entry);
    let entry_name = entry_path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("bundle");
    let hash = hash_string(&bundle_code);

    // Write JavaScript bundle
    let js_name = format!("{}-{}.js", entry_name, hash);
    let js_path = outdir_path.join(&js_name);

    // Add source map URL comment if source map exists
    let mut final_code = bundle_code;
    if source_map.is_some() {
        let map_name = format!("{}-{}.js.map", entry_name, hash);
        final_code.push_str(&format!("\n//# sourceMappingURL={}\n", map_name));
    }

    std::fs::write(&js_path, &final_code)
        .map_err(|e| format!("Failed to write JS bundle: {}", e))?;

    // Write source map if present
    if let Some(map_content) = source_map {
        let map_name = format!("{}-{}.js.map", entry_name, hash);
        let map_path = outdir_path.join(&map_name);
        std::fs::write(&map_path, map_content)
            .map_err(|e| format!("Failed to write source map: {}", e))?;
        stdio_log::debug("sourcemap", &format!("written to {}", map_name));
    }

    // Write CSS if present
    let mut css_name = None;
    if let Some(css_content) = css_code {
        let css_hash = hash_string(&css_content);
        let css_file = format!("{}-{}.css", entry_name, css_hash);
        css_name = Some(css_file.clone());
        let css_path = outdir_path.join(&css_file);
        std::fs::write(&css_path, css_content)
            .map_err(|e| format!("Failed to write CSS: {}", e))?;
    }

    // Write HTML wrapper
    let html_name = format!("{}.html", entry_name);
    let html_path = outdir_path.join(&html_name);
    let html = build_html_wrapper(&js_name, css_name.as_deref());
    std::fs::write(&html_path, html)
        .map_err(|e| format!("Failed to write HTML: {}", e))?;

    let duration = start.elapsed().as_millis();
    stdio_log::log("build", &format!("complete in {}ms", duration));
    Ok(())
}

fn resolve_entry_path(entry: &str) -> Result<String, String> {
    let path = PathBuf::from(entry);
    if path.is_absolute() {
        return Ok(entry.to_string());
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    Ok(cwd.join(path).display().to_string())
}

fn hash_string(s: &str) -> String {
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
    }
    format!("{:x}", hash)
}

fn build_html_wrapper(js_name: &str, css_name: Option<&str>) -> String {
    let css_link = css_name
        .map(|name| format!("    <link rel=\"stylesheet\" href=\"./{}\">\n", name))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Deka App</title>
{css_link}  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="./{}"></script>
  </body>
</html>
"#,
        js_name
    )
}
