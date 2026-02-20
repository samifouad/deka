use core::{CommandSpec, Context, Registry};
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
        "{{\n  \"name\": \"{}\",\n  \"type\": \"serve\",\n  \"serve\": {{ \"entry\": \"app/main.phpx\" }},\n  \"scripts\": {{ \"dev\": \"deka serve --dev\" }},\n  \"security\": {{\n    \"allow\": {{\n      \"read\": [\".\"],\n      \"write\": [\".cache\", \"php_modules/.cache\"],\n      \"wasm\": [\"*\"],\n      \"env\": [\"*\"]\n    }}\n  }}\n}}\n",
        name
    )
}

fn default_deka_lock_json() -> String {
    "{\n  \"lockfileVersion\": 1,\n  \"node\": {\n    \"packages\": {}\n  },\n  \"php\": {\n    \"packages\": {},\n    \"cache\": {\n      \"version\": 1,\n      \"compiler\": \"phpx-cache-v3\",\n      \"modules\": {}\n    }\n  }\n}\n".to_string()
}

fn default_main_phpx() -> &'static str {
    "---\n$title = \"Deka App\"\n---\n<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <title>{$title}</title>\n  </head>\n  <body>\n    <main>\n      <h1>{$title}</h1>\n      <p>Project initialized. Edit <code>app/main.phpx</code>.</p>\n    </main>\n  </body>\n</html>\n"
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
    "export function renderToString($node) {
    if (is_string($node)) {
        return $node;
    }
    return \"\";
}

export function Hydration($props: object): mixed {
    return null;
}
"
    .to_string()
}
