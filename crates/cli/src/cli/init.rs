use core::{CommandSpec, Context, Registry};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use stdio::{error as stdio_error, raw};

const COMMAND: CommandSpec = CommandSpec {
    name: "init",
    category: "project",
    summary: "initialize a new app project",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

pub fn cmd(context: &Context) {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            stdio_error(
                "init",
                &format!("failed to resolve current directory: {}", err),
            );
            return;
        }
    };

    let target = if let Some(dir) = context.args.positionals.first() {
        cwd.join(dir)
    } else {
        cwd
    };

    if let Err(err) = std::fs::create_dir_all(&target) {
        stdio_error(
            "init",
            &format!("failed to create {}: {}", target.display(), err),
        );
        return;
    }

    let mut touched: Vec<String> = Vec::new();

    if let Err(err) = ensure_file(
        &target.join("deka.json"),
        default_deka_json(project_name_from_dir(&target).as_str()),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &target.join("deka.lock"),
        default_deka_lock_json(),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = std::fs::create_dir_all(target.join("app")) {
        stdio_error("init", &format!("failed to create app/: {}", err));
        return;
    }
    if let Err(err) = ensure_file(
        &target.join("app").join("main.phpx"),
        default_main_phpx().to_string(),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }
    if let Err(err) = ensure_file(
        &target.join("app").join("page.phpx"),
        default_app_page_phpx().to_string(),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }
    if let Err(err) = ensure_file(
        &target.join("app").join("layout.phpx"),
        default_app_layout_phpx().to_string(),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = std::fs::create_dir_all(target.join("public")) {
        stdio_error("init", &format!("failed to create public/: {}", err));
        return;
    }
    if let Err(err) = ensure_file(
        &target.join("public").join("index.html"),
        default_public_index_html().to_string(),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    let php_modules = target.join("php_modules");
    if let Err(err) = std::fs::create_dir_all(&php_modules) {
        stdio_error("init", &format!("failed to create php_modules/: {}", err));
        return;
    }

    let deka_php = php_modules.join("deka.php");
    if !deka_php.exists() {
        let template = load_deka_php_template().unwrap_or_else(default_deka_php);
        if let Err(err) = std::fs::write(&deka_php, template.as_bytes()) {
            stdio_error(
                "init",
                &format!("failed to write {}: {}", deka_php.display(), err),
            );
            return;
        }
        touched.push(path_display(&deka_php));
    }

    if let Err(err) = ensure_file(
        &php_modules.join("component").join("core.phpx"),
        load_module_template("component/core.phpx").unwrap_or_else(default_component_core_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("component").join("dom.phpx"),
        load_module_template("component/dom.phpx").unwrap_or_else(default_component_dom_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }
    if let Err(err) = ensure_file(
        &php_modules.join("component").join("router.phpx"),
        load_module_template("component/router.phpx").unwrap_or_else(default_component_router_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("encoding").join("json").join("index.phpx"),
        load_module_template("encoding/json/index.phpx")
            .unwrap_or_else(default_encoding_json_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("fs").join("index.phpx"),
        load_module_template("fs/index.phpx").unwrap_or_else(default_fs_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("time").join("index.phpx"),
        load_module_template("time/index.phpx").unwrap_or_else(default_time_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("core").join("result.phpx"),
        load_module_template("core/result.phpx").unwrap_or_else(default_core_result_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("core").join("byte.phpx"),
        load_module_template("core/byte.phpx").unwrap_or_else(default_core_byte_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("core").join("bytes.phpx"),
        load_module_template("core/bytes.phpx").unwrap_or_else(default_core_bytes_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("core").join("num.phpx"),
        load_module_template("core/num.phpx").unwrap_or_else(default_core_num_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_file(
        &php_modules.join("core").join("bridge.phpx"),
        load_module_template("core/bridge.phpx").unwrap_or_else(default_core_bridge_phpx),
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if let Err(err) = ensure_lock_module_entries(
        &target.join("deka.lock"),
        &[
            ("component/core", "php_modules/component/core.phpx"),
            ("component/dom", "php_modules/component/dom.phpx"),
            ("component/router", "php_modules/component/router.phpx"),
            ("encoding/json", "php_modules/encoding/json/index.phpx"),
            ("fs", "php_modules/fs/index.phpx"),
            ("time", "php_modules/time/index.phpx"),
            ("core/result", "php_modules/core/result.phpx"),
            ("core/byte", "php_modules/core/byte.phpx"),
            ("core/bytes", "php_modules/core/bytes.phpx"),
            ("core/num", "php_modules/core/num.phpx"),
            ("core/bridge", "php_modules/core/bridge.phpx"),
        ],
        &mut touched,
    ) {
        stdio_error("init", &err);
        return;
    }

    if touched.is_empty() {
        raw("[init] project is already initialized");
        return;
    }

    raw("[init] initialized project files:");
    for path in touched {
        raw(&format!("  - {}", path));
    }
}

fn ensure_file(path: &Path, content: String, touched: &mut Vec<String>) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {}", parent.display(), err))?;
    }
    std::fs::write(path, content.as_bytes())
        .map_err(|err| format!("failed to write {}: {}", path.display(), err))?;
    touched.push(path_display(path));
    Ok(())
}

fn ensure_lock_module_entries(
    lock_path: &Path,
    entries: &[(&str, &str)],
    touched: &mut Vec<String>,
) -> Result<(), String> {
    let mut doc = if lock_path.exists() {
        let raw = std::fs::read_to_string(lock_path)
            .map_err(|err| format!("failed to read {}: {}", lock_path.display(), err))?;
        serde_json::from_str::<Value>(&raw)
            .map_err(|err| format!("invalid JSON in {}: {}", lock_path.display(), err))?
    } else {
        serde_json::from_str::<Value>(&default_deka_lock_json())
            .map_err(|err| format!("invalid default deka.lock JSON: {}", err))?
    };

    let root = doc
        .as_object_mut()
        .ok_or("deka.lock must be a JSON object")?;
    let php = root
        .entry("php")
        .or_insert_with(|| Value::Object(Map::new()));
    let php_obj = php
        .as_object_mut()
        .ok_or("deka.lock php section must be an object")?;
    let cache = php_obj
        .entry("cache")
        .or_insert_with(|| Value::Object(Map::new()));
    let cache_obj = cache
        .as_object_mut()
        .ok_or("deka.lock php.cache must be an object")?;
    cache_obj
        .entry("version")
        .or_insert_with(|| Value::Number(1.into()));
    cache_obj
        .entry("compiler")
        .or_insert_with(|| Value::String("phpx-cache-v3".to_string()));
    let modules = cache_obj
        .entry("modules")
        .or_insert_with(|| Value::Object(Map::new()));
    let modules_obj = modules
        .as_object_mut()
        .ok_or("deka.lock php.cache.modules must be an object")?;

    let mut changed = false;
    for (module_id, src_rel) in entries {
        if modules_obj.contains_key(*module_id) {
            continue;
        }
        let cache_rel = module_cache_rel(src_rel);
        let entry = serde_json::json!({
            "src": src_rel,
            "hash": "",
            "cache": cache_rel,
            "compiler": "phpx-cache-v3",
            "deps": [],
            "exports": []
        });
        modules_obj.insert((*module_id).to_string(), entry);
        changed = true;
    }

    if changed {
        let payload = serde_json::to_string_pretty(&doc)
            .map_err(|err| format!("failed to serialize {}: {}", lock_path.display(), err))?;
        std::fs::write(lock_path, payload)
            .map_err(|err| format!("failed to write {}: {}", lock_path.display(), err))?;
        touched.push(path_display(lock_path));
    }

    Ok(())
}

fn module_cache_rel(src_rel: &str) -> String {
    let normalized = src_rel.replace('\\', "/");
    let trimmed = normalized.strip_prefix("php_modules/").unwrap_or(normalized.as_str());
    let mut rel = trimmed.to_string();
    if rel.ends_with(".phpx") {
        rel.truncate(rel.len() - 5);
        rel.push_str(".php");
    }
    format!("php_modules/.cache/phpx/{}", rel)
}

fn project_name_from_dir(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("app")
        .to_string()
}

fn path_display(path: &Path) -> String {
    match std::env::current_dir() {
        Ok(cwd) => match path.strip_prefix(&cwd) {
            Ok(rel) => rel.display().to_string(),
            Err(_) => path.display().to_string(),
        },
        Err(_) => path.display().to_string(),
    }
}

fn default_deka_json(name: &str) -> String {
    format!(
        "{{\n  \"name\": \"{}\",\n  \"type\": \"serve\",\n  \"serve\": {{ \"entry\": \"app/main.phpx\" }},\n  \"scripts\": {{ \"dev\": \"deka serve --dev\" }},\n  \"security\": {{\n    \"allow\": {{\n      \"read\": [\".\"],\n      \"write\": [\".cache\", \"php_modules/.cache\"],\n      \"wasm\": [\"*\"],\n      \"env\": [\"*\"],\n      \"db\": [\"stats\"]\n    }}\n  }}\n}}\n",
        name
    )
}

fn default_deka_lock_json() -> String {
    "{\n  \"lockfileVersion\": 1,\n  \"node\": {\n    \"packages\": {}\n  },\n  \"php\": {\n    \"packages\": {},\n    \"cache\": {\n      \"version\": 1,\n      \"compiler\": \"phpx-cache-v3\",\n      \"modules\": {}\n    }\n  }\n}\n".to_string()
}

fn default_main_phpx() -> &'static str {
    "import { Router } from 'component/router';\n\necho Router();\n"
}

fn default_app_page_phpx() -> &'static str {
    "export function Page() {\n    return <div>\n        <h1>Deka App</h1>\n        <p>Project initialized. Edit <code>app/page.phpx</code>.</p>\n    </div>;\n}\n"
}

fn default_app_layout_phpx() -> &'static str {
    "export function Layout($props) {\n    return <html lang=\"en\">\n      <head>\n        <meta charset=\"utf-8\" />\n        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n        <title>Deka App</title>\n      </head>\n      <body>\n        <main>{$props.children}</main>\n      </body>\n    </html>;\n}\n"
}

fn default_public_index_html() -> &'static str {
    "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <title>Deka</title>\n  </head>\n  <body>\n    <div id=\"app\"></div>\n  </body>\n</html>\n"
}

fn load_deka_php_template() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(root) = std::env::var("PHPX_MODULE_ROOT") {
        let root = PathBuf::from(root);
        if root.join("deka.lock").is_file() {
            candidates.push(root.join("php_modules").join("deka.php"));
            candidates.push(root.join("deka.php"));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut cursor = exe.parent().map(PathBuf::from);
        for _ in 0..6 {
            if let Some(dir) = cursor.clone() {
                candidates.push(dir.join("php_modules").join("deka.php"));
                cursor = dir.parent().map(PathBuf::from);
            } else {
                break;
            }
        }
    }

    for candidate in candidates {
        if let Ok(content) = std::fs::read_to_string(&candidate) {
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    None
}

fn load_module_template(module_rel: &str) -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(root) = std::env::var("PHPX_MODULE_ROOT") {
        let root = PathBuf::from(root);
        if root.join("deka.lock").is_file() {
            candidates.push(root.join("php_modules").join(module_rel));
            candidates.push(root.join(module_rel));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut cursor = exe.parent().map(PathBuf::from);
        for _ in 0..6 {
            if let Some(dir) = cursor.clone() {
                candidates.push(dir.join("php_modules").join(module_rel));
                cursor = dir.parent().map(PathBuf::from);
            } else {
                break;
            }
        }
    }

    for candidate in candidates {
        if let Ok(content) = std::fs::read_to_string(&candidate) {
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    None
}

fn default_deka_php() -> String {
    "<?php\n// Minimal runtime prelude generated by `deka init`.\n\nif (!function_exists('panic')) {\n    function panic(string $message): void {\n        throw new \\Exception($message);\n    }\n}\n\n$GLOBALS['__DEKA_PHPX_STDLIB'] = $GLOBALS['__DEKA_PHPX_STDLIB'] ?? [];\n".to_string()
}

fn default_component_core_phpx() -> String {
    "export function jsx($type, $props = false, ...$children): object {
    return { type: $type, props: $props, children: $children };
}

export function jsxs($type, $props = false, ...$children): object {
    return jsx($type, $props, ...$children);
}
"
    .to_string()
}

fn default_component_dom_phpx() -> String {
    r#"export function renderToString($node) {
    if (is_string($node)) {
        return $node;
    }
    return "";
}

export function Hydration($props: object): mixed {
    return null;
}
"#
        .to_string()
}

fn default_component_router_phpx() -> String {
    r#"export function generate_manifest($root = '.') {
    return { ok: false, error: 'router manifest unavailable' };
}
"#
        .to_string()
}

fn default_encoding_json_phpx() -> String {
    r#"import { result_ok, result_err } from 'core/result';
import { byte_is_digit, byte_is_hex, byte_is_whitespace } from 'core/byte';
import { num_parse_int, num_parse_float } from 'core/num';
import { bridge } from 'core/bridge';

export function json_encode($value) {
    return bridge('json', 'encode', { value: $value });
}

export function json_decode($value, $assoc = false) {
    return bridge('json', 'decode', { value: $value, assoc: $assoc });
}

export function json_decode_result($value, $assoc = false): Result {
    return json_decode($value, $assoc);
}

export function json_last_error(): int {
    return 0;
}

export function json_last_error_msg(): string {
    return '';
}

export function json_validate($value): bool {
    return true;
}
"# 
        .to_string()
}

fn default_fs_phpx() -> String {
    r#"import { result_err } from 'core/result';

export function readDirSync($path) {
    return result_err('fs unavailable');
}

export function readDir($path) {
    return readDirSync($path);
}

export function readFileSync($path) {
    return result_err('fs unavailable');
}

export function readFile($path) {
    return readFileSync($path);
}

export function writeFileSync($path, $bytes) {
    return result_err('fs unavailable');
}

export function writeFile($path, $bytes) {
    return writeFileSync($path, $bytes);
}

export function mkdirSync($path) {
    return result_err('fs unavailable');
}

export function mkdir($path) {
    return mkdirSync($path);
}
"#
    .to_string()
}

fn default_time_phpx() -> String {
    r#"export function now_ms() {
    return 0;
}
"#
    .to_string()
}

fn default_core_result_phpx() -> String {
    r#"enum Result {
    case Ok(mixed $value);
    case Err(mixed $error);
}

export function result_ok($value): Result {
    return Result::Ok($value);
}

export function result_err($error): Result {
    return Result::Err($error);
}

export function result_is_ok($value): bool {
    return $value is Result::Ok;
}

export function result_is_err($value): bool {
    return $value is Result::Err;
}

export function result_unwrap($value) {
    if ($value is Result::Ok) return $value.value;
    panic('Tried to unwrap Err result.');
}

export function result_unwrap_or($value, $fallback) {
    if ($value is Result::Ok) return $value.value;
    return $fallback;
}
"#
        .to_string()
}

fn default_core_byte_phpx() -> String {
    r#"export function byte_from_char($char): int {
    return ord('' . $char);
}

export function byte_to_char($byte): string {
    return chr($byte);
}

export function byte_is_whitespace($byte): bool {
    return $byte === 9 || $byte === 10 || $byte === 13 || $byte === 32;
}

export function byte_is_digit($byte): bool {
    return $byte >= 48 && $byte <= 57;
}

export function byte_is_hex($byte): bool {
    return ($byte >= 48 && $byte <= 57) || ($byte >= 65 && $byte <= 70) || ($byte >= 97 && $byte <= 102);
}
"# 
        .to_string()
}

fn default_core_bytes_phpx() -> String {
    r#"export function bytes_from_array($arr) {
    return '';
}

export function bytes_to_array($bytes) {
    return [];
}
"#
        .to_string()
}

fn default_core_num_phpx() -> String {
    r#"export function num_parse_int($value): int {
    return intval($value);
}

export function num_parse_float($value): float {
    return floatval($value);
}
"#
        .to_string()
}

fn default_core_bridge_phpx() -> String {
    r#"export function bridge($kind, $action, $payload = {}) {
    if (function_exists('__bridge')) {
        return \__bridge($kind, $action, $payload);
    }
    return \__deka_wasm_call('__deka_' . ('' . $kind), '' . $action, $payload);
}

export async function bridge_async($kind, $action, $payload = {}): Promise<mixed> {
    if (function_exists('__bridge_async')) {
        return await \__bridge_async($kind, $action, $payload);
    }
    return await \__deka_wasm_call_async('__deka_' . ('' . $kind), '' . $action, $payload);
}
"#
        .to_string()
}
