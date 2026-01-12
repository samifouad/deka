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
    // Use parallel bundler if DEKA_PARALLEL_BUNDLER=1
    let use_parallel = std::env::var("DEKA_PARALLEL_BUNDLER")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    let bundle_code = if use_parallel {
        eprintln!(" using parallel bundler ({} workers)", num_cpus::get());
        let root = PathBuf::from(".").canonicalize()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let bundler = bundler::ParallelBundler::new(root);
        bundler.bundle(&entry_path).await?
    } else {
        let bundle = bundler::bundle_browser_assets(&entry_path)
            .map_err(|e| format!("Bundle failed: {}", e))?;
        bundle.code
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
    std::fs::write(&js_path, &bundle_code)
        .map_err(|e| format!("Failed to write JS bundle: {}", e))?;

    // Write CSS if present (only with standard bundler for now)
    let mut css_name = None;
    if !use_parallel {
        let bundle = bundler::bundle_browser_assets(&entry_path)
            .map_err(|e| format!("Bundle failed: {}", e))?;
        if let Some(css_code) = bundle.css {
            let css_hash = hash_string(&css_code);
            let css_file = format!("{}-{}.css", entry_name, css_hash);
            css_name = Some(css_file.clone());
            let css_path = outdir_path.join(&css_file);
            std::fs::write(&css_path, css_code)
                .map_err(|e| format!("Failed to write CSS: {}", e))?;
        }
    }

    // Write HTML wrapper
    let html_name = format!("{}.html", entry_name);
    let html_path = outdir_path.join(&html_name);
    let html = build_html_wrapper(&js_name, css_name.as_deref());
    std::fs::write(&html_path, html)
        .map_err(|e| format!("Failed to write HTML: {}", e))?;

    let duration = start.elapsed().as_millis();
    eprintln!(" build complete [{}ms]", duration);
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
