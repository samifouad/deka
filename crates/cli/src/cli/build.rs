use bumpalo::Bump;
use core::{CommandSpec, Context, ParamSpec, Registry};
use modules_php::compiler_api::compile_phpx;
use modules_php::validation::format_multiple_errors;
use php_rs::parser::ast::{
    BinaryOp, ClassKind, ClassMember, Expr, ExprId, JsxChild, ObjectKey, Program, Stmt, StmtId,
    Type as AstType, UnaryOp,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const COMMAND: CommandSpec = CommandSpec {
    name: "build",
    category: "project",
    summary: "build a PHPX file into a JavaScript module (JS runtime semantics)",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
    registry.add_param(ParamSpec {
        name: "--out",
        description: "output JavaScript file path",
    });
}

pub fn cmd(context: &Context) {
    if let Err(err) = run(context) {
        stdio::error("build", &err);
    }
}

fn run(context: &Context) -> Result<(), String> {
    if let Some(first) = context.args.positionals.first() {
        if first.ends_with(".phpx") {
            return run_single_file_build(context, first);
        }
    }

    run_web_project_build(context)
}

fn run_single_file_build(context: &Context, input: &str) -> Result<(), String> {
    if !input.ends_with(".phpx") {
        return Err(format!(
            "build currently supports .phpx input only; got '{}'",
            input
        ));
    }

    let input_path = PathBuf::from(input);
    let output_path = resolve_output_path(output_arg(context), &input_path)?;
    build_single_file_to_path(&input_path, &output_path)?;

    stdio::success(&format!(
        "built {} -> {}",
        input_path.display(),
        output_path.display()
    ));
    Ok(())
}

fn run_web_project_build(context: &Context) -> Result<(), String> {
    let root_hint = context
        .args
        .positionals
        .first()
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().map_err(|err| err.to_string())?);

    let project_root = resolve_project_root(&root_hint)?;
    ensure_web_project_layout(&project_root)?;

    let app_dir = project_root.join("app");
    let public_dir = project_root.join("public");
    let entry_path = resolve_web_entry(&project_root)?;

    let dist_root = project_root.join("dist");
    let dist_client = dist_root.join("client");
    let dist_server = dist_root.join("server");
    let dist_assets = dist_client.join("assets");

    fs::create_dir_all(&dist_assets)
        .map_err(|err| format!("failed to create {}: {}", dist_assets.display(), err))?;
    fs::create_dir_all(&dist_server)
        .map_err(|err| format!("failed to create {}: {}", dist_server.display(), err))?;

    copy_dir_recursive(&public_dir, &dist_client)?;

    let client_js = dist_assets.join("main.js");
    build_single_file_to_path(&entry_path, &client_js)?;

    let assets_importmap = dist_assets.join("importmap.json");
    let client_importmap = dist_client.join("importmap.json");
    if assets_importmap.is_file() {
        fs::copy(&assets_importmap, &client_importmap).map_err(|err| {
            format!(
                "failed to copy {} -> {}: {}",
                assets_importmap.display(),
                client_importmap.display(),
                err
            )
        })?;
    }

    let client_index = dist_client.join("index.html");
    let index_raw = fs::read_to_string(&client_index)
        .map_err(|err| format!("failed to read {}: {}", client_index.display(), err))?;
    let index_out = inject_web_bootstrap_tags(&index_raw);
    fs::write(&client_index, index_out)
        .map_err(|err| format!("failed to write {}: {}", client_index.display(), err))?;

    copy_dir_recursive(&app_dir, &dist_server.join("app"))?;

    let modules_dir = project_root.join("php_modules");
    if modules_dir.is_dir() {
        copy_dir_recursive(&modules_dir, &dist_server.join("php_modules"))?;
    }
    for file in ["deka.json", "deka.lock"] {
        let src = project_root.join(file);
        if src.is_file() {
            let dst = dist_server.join(file);
            fs::copy(&src, &dst).map_err(|err| {
                format!("failed to copy {} -> {}: {}", src.display(), dst.display(), err)
            })?;
        }
    }

    stdio::success(&format!(
        "built web project {}\n  client: {}\n  server: {}",
        project_root.display(),
        dist_client.display(),
        dist_server.display()
    ));
    Ok(())
}

fn output_arg(context: &Context) -> Option<String> {
    context
        .args
        .params
        .get("--out")
        .or_else(|| context.args.params.get("-o"))
        .or_else(|| context.args.params.get("--outdir"))
        .cloned()
}

fn resolve_output_path(out: Option<String>, input_path: &Path) -> Result<PathBuf, String> {
    if let Some(out) = out {
        return Ok(PathBuf::from(out));
    }

    let stem = input_path
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or_else(|| format!("invalid input filename: {}", input_path.display()))?;

    Ok(PathBuf::from("dist").join(format!("{}.js", stem)))
}

fn resolve_import_map_path(output_path: &Path) -> PathBuf {
    output_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("importmap.json")
}

fn emit_import_map_json(meta: &SourceModuleMeta, output_path: &Path) -> String {
    let mut imports = default_import_map();

    for decl in &meta.imports {
        let spec = decl.from.trim();
        if !is_bare_specifier(spec) {
            continue;
        }

        if !imports.contains_key(spec) && !is_covered_by_prefix_map(&imports, spec) {
            imports.insert(spec.to_string(), default_import_target_for(spec, output_path));
        }
    }

    serde_json::to_string_pretty(&serde_json::json!({ "imports": imports }))
        .unwrap_or_else(|_| "{\n  \"imports\": {}\n}".to_string())
}

fn default_import_map() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("@/".to_string(), "/".to_string()),
        ("component/".to_string(), "/php_modules/component/".to_string()),
        ("deka/".to_string(), "/php_modules/deka/".to_string()),
        ("encoding/".to_string(), "/php_modules/encoding/".to_string()),
        ("db/".to_string(), "/php_modules/db/".to_string()),
    ])
}

fn is_bare_specifier(spec: &str) -> bool {
    !spec.is_empty()
        && !spec.starts_with("./")
        && !spec.starts_with("../")
        && !spec.starts_with('/')
        && !spec.starts_with("http://")
        && !spec.starts_with("https://")
}

fn is_covered_by_prefix_map(map: &BTreeMap<String, String>, spec: &str) -> bool {
    map.keys()
        .any(|key| key.ends_with('/') && spec.starts_with(key))
}

fn default_import_target_for(spec: &str, output_path: &Path) -> String {
    let mut rel = String::new();
    let depth = output_path
        .parent()
        .map(|parent| parent.components().count().saturating_sub(1))
        .unwrap_or(0);

    for _ in 0..depth {
        rel.push_str("../");
    }
    rel.push_str("php_modules/");
    rel.push_str(spec.trim_start_matches('/'));

    if !rel.ends_with(".js") && !rel.ends_with('/') {
        rel.push_str(".js");
    }

    rel
}

fn resolve_project_root(input_path: &Path) -> Result<PathBuf, String> {
    let start = if input_path.is_dir() {
        input_path.to_path_buf()
    } else {
        input_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
    };

    for dir in start.ancestors() {
        if dir.join("deka.json").is_file() {
            return Ok(dir.to_path_buf());
        }
    }

    Err(format!(
        "deka build requires a deka.json project root (searched from {})",
        input_path.display()
    ))
}

fn ensure_project_layout(project_root: &Path, meta: &SourceModuleMeta) -> Result<(), String> {
    let lock_path = project_root.join("deka.lock");
    if !lock_path.is_file() {
        return Err(format!(
            "deka build requires deka.lock at project root: {}",
            lock_path.display()
        ));
    }

    let stdlib_imports = collect_stdlib_imports(meta);
    if stdlib_imports.is_empty() {
        return Ok(());
    }

    let modules_dir = project_root.join("php_modules");
    if !modules_dir.is_dir() {
        return Err(format!(
            "deka build requires php_modules/ at project root when using stdlib imports ({})",
            stdlib_imports.join(", ")
        ));
    }

    let mut missing = Vec::new();
    for spec in stdlib_imports {
        if resolve_module_file(&modules_dir, &spec).is_none() {
            missing.push(spec);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing stdlib modules under {}: {}",
            modules_dir.display(),
            missing.join(", ")
        ))
    }
}

fn collect_stdlib_imports(meta: &SourceModuleMeta) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for decl in &meta.imports {
        let spec = decl.from.trim();
        if is_stdlib_module_spec(spec) {
            seen.insert(spec.to_string());
        }
    }
    seen.into_iter().collect()
}

fn is_stdlib_module_spec(spec: &str) -> bool {
    if !is_bare_specifier(spec) || spec.starts_with("@user/") {
        return false;
    }

    spec.starts_with("component/")
        || spec.starts_with("deka/")
        || spec.starts_with("encoding/")
        || spec.starts_with("db/")
        || matches!(
            spec,
            "json"
                | "postgres"
                | "mysql"
                | "sqlite"
                | "bytes"
                | "buffer"
                | "tcp"
                | "tls"
                | "fs"
                | "crypto"
                | "jwt"
                | "cookies"
                | "auth"
                | "db"
        )
}

fn resolve_module_file(modules_dir: &Path, spec: &str) -> Option<PathBuf> {
    let mut candidates = vec![
        modules_dir.join(format!("{}.phpx", spec)),
        modules_dir.join(format!("{}.php", spec)),
        modules_dir.join(spec).join("index.phpx"),
        modules_dir.join(spec).join("index.php"),
    ];

    if spec.ends_with(".phpx") || spec.ends_with(".php") {
        candidates.insert(0, modules_dir.join(spec));
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn load_deka_json(project_root: &Path) -> Result<serde_json::Value, String> {
    let deka_path = project_root.join("deka.json");
    let raw = fs::read_to_string(&deka_path)
        .map_err(|err| format!("failed to read {}: {}", project_root.join("deka.json").display(), err))?;
    serde_json::from_str(&raw).map_err(|err| format!("invalid {}: {}", project_root.join("deka.json").display(), err))
}

fn ensure_web_project_layout(project_root: &Path) -> Result<(), String> {
    let required_files = [project_root.join("deka.json"), project_root.join("deka.lock")];
    for file in &required_files {
        if !file.is_file() {
            return Err(format!("missing required file: {}", file.display()));
        }
    }

    let required_dirs = [project_root.join("app"), project_root.join("public")];
    for dir in &required_dirs {
        if !dir.is_dir() {
            return Err(format!("missing required directory: {}", dir.display()));
        }
    }

    let index = project_root.join("public").join("index.html");
    if !index.is_file() {
        return Err(format!("missing required file: {}", index.display()));
    }

    let json = load_deka_json(project_root)?;
    let project_type = json
        .get("type")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_ascii_lowercase());

    if project_type.as_deref() != Some("serve") {
        let got = project_type.unwrap_or_else(|| "<missing>".to_string());
        return Err(format!(
            "web build requires deka.json type=\"serve\" (got: {}) at {}",
            got,
            project_root.join("deka.json").display()
        ));
    }

    Ok(())
}

fn resolve_web_entry(project_root: &Path) -> Result<PathBuf, String> {
    let json = load_deka_json(project_root)?;

    let entry = json
        .get("serve")
        .and_then(|v| v.get("entry"))
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "web build requires deka.json serve.entry (example: \"app/main.phpx\") in {}",
                project_root.join("deka.json").display()
            )
        })?;

    let entry_path = project_root.join(entry);
    if !entry_path.is_file() {
        return Err(format!(
            "serve.entry points to missing file: {}",
            entry_path.display()
        ));
    }

    let app_dir = project_root.join("app");
    if !entry_path.starts_with(&app_dir) {
        return Err(format!(
            "serve.entry must point inside app/: {}",
            entry_path.display()
        ));
    }

    if entry_path.extension().and_then(|e| e.to_str()) != Some("phpx") {
        return Err(format!(
            "serve.entry must be a .phpx file: {}",
            entry_path.display()
        ));
    }

    Ok(entry_path)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|err| format!("failed to create {}: {}", dst.display(), err))?;
    let entries = fs::read_dir(src).map_err(|err| format!("failed to read {}: {}", src.display(), err))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry error: {}", err))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|err| format!("file_type error for {}: {}", src_path.display(), err))?;

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("failed to create {}: {}", parent.display(), err))?;
            }
            fs::copy(&src_path, &dst_path).map_err(|err| {
                format!(
                    "failed to copy {} -> {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    err
                )
            })?;
        }
    }

    Ok(())
}

fn inject_web_bootstrap_tags(index_html: &str) -> String {
    let import_map_tag = r#"<script type="importmap" src="/importmap.json"></script>"#;
    let module_tag = r#"<script type="module" src="/assets/main.js"></script>"#;

    let mut out = index_html.to_string();

    if !out.contains(import_map_tag) {
        if out.contains("</head>") {
            out = out.replace("</head>", &format!("  {}\n</head>", import_map_tag));
        } else {
            out.push('\n');
            out.push_str(import_map_tag);
            out.push('\n');
        }
    }

    if !out.contains(module_tag) {
        if out.contains("</body>") {
            out = out.replace("</body>", &format!("  {}\n</body>", module_tag));
        } else {
            out.push('\n');
            out.push_str(module_tag);
            out.push('\n');
        }
    }

    out
}

fn build_single_file_to_path(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let input = input_path
        .to_str()
        .ok_or_else(|| format!("invalid utf-8 path: {}", input_path.display()))?;

    let source = fs::read_to_string(input_path)
        .map_err(|err| format!("failed to read {}: {}", input_path.display(), err))?;
    let meta = parse_source_module_meta(&source);

    let project_root = resolve_project_root(input_path)?;
    ensure_project_layout(&project_root, &meta)?;

    let arena = Bump::new();
    let result = compile_phpx(&source, input, &arena);
    if !result.errors.is_empty() {
        let formatted = format_multiple_errors(&source, input, &result.errors, &result.warnings);
        return Err(formatted);
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {}", parent.display(), err))?;
    }

    let js = if let Some(program) = result.ast {
        match emit_js_from_ast(&program, source.as_bytes(), meta.clone()) {
            Ok(emitted) => emitted,
            Err(reason) => emit_js_scaffold_with_reason(&source, input, &reason),
        }
    } else {
        emit_js_scaffold_with_reason(&source, input, "no AST available after validation")
    };

    fs::write(output_path, js)
        .map_err(|err| format!("failed to write {}: {}", output_path.display(), err))?;

    let import_map_path = resolve_import_map_path(output_path);
    let import_map = emit_import_map_json(&meta, output_path);
    fs::write(&import_map_path, import_map)
        .map_err(|err| format!("failed to write {}: {}", import_map_path.display(), err))?;

    Ok(())
}

fn emit_js_from_ast(
    program: &Program<'_>,
    source: &[u8],
    meta: SourceModuleMeta,
) -> Result<String, String> {
    let mut emitter = JsSubsetEmitter::new(source, meta);
    emitter.emit_program(program)?;
    Ok(emitter.finish())
}

fn emit_js_scaffold_with_reason(source: &str, file_path: &str, reason: &str) -> String {
    let escaped = serde_json::to_string(source).unwrap_or_else(|_| "\"\"".to_string());
    let escaped_path =
        serde_json::to_string(file_path).unwrap_or_else(|_| "\"unknown.phpx\"".to_string());
    let escaped_reason =
        serde_json::to_string(reason).unwrap_or_else(|_| "\"unknown\"".to_string());

    format!(
        "// Generated by deka build. Do not edit manually.\n\
// Source: {file_path}\n\
// Target semantics: JavaScript runtime semantics.\n\
// Fallback scaffold used because subset emitter could not lower this file.\n\
export const phpxBuildMode = \"scaffold\";\n\
export const phpxTargetSemantics = \"js\";\n\
export const phpxBuildReason = {escaped_reason};\n\
export const phpxSource = {escaped};\n\
export const phpxFile = {escaped_path};\n\
\n\
export async function runPhpx(runtime, props = {{}}) {{\n\
  if (!runtime || typeof runtime.executePhpx !== 'function') {{\n\
    throw new Error('runtime.executePhpx(source, file, props) is required');\n\
  }}\n\
  return await runtime.executePhpx(phpxSource, phpxFile, props);\n\
}}\n",
    )
}

#[derive(Debug, Clone)]
struct ImportSpec {
    imported: String,
    local: String,
}

#[derive(Debug, Clone)]
struct ImportDecl {
    from: String,
    specs: Vec<ImportSpec>,
}

#[derive(Debug, Clone)]
struct SourceModuleMeta {
    imports: Vec<ImportDecl>,
    exported_functions: HashSet<String>,
    export_specs: Vec<ImportSpec>,
}

impl SourceModuleMeta {
    fn empty() -> Self {
        Self {
            imports: Vec::new(),
            exported_functions: HashSet::new(),
            export_specs: Vec::new(),
        }
    }
}

fn parse_source_module_meta(source: &str) -> SourceModuleMeta {
    let mut meta = SourceModuleMeta::empty();
    let lines: Vec<&str> = source.lines().collect();
    let (start, end) = frontmatter_range(&lines).unwrap_or((0, lines.len()));

    for line in &lines[start..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if let Some(decl) = parse_import_line(trimmed) {
            meta.imports.push(decl);
            continue;
        }

        if let Some(name) = parse_export_function_line(trimmed) {
            meta.exported_functions.insert(name);
            continue;
        }

        if let Some(specs) = parse_export_specs_line(trimmed) {
            meta.export_specs.extend(specs);
        }
    }

    meta
}

fn frontmatter_range(lines: &[&str]) -> Option<(usize, usize)> {
    let mut first = None;
    let mut second = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim() == "---" {
            if first.is_none() {
                first = Some(idx);
            } else {
                second = Some(idx);
                break;
            }
        }
    }
    match (first, second) {
        (Some(a), Some(b)) if b > a => Some((a + 1, b)),
        _ => None,
    }
}

fn parse_import_line(line: &str) -> Option<ImportDecl> {
    let trimmed = line.trim_end_matches(';').trim();
    if !trimmed.starts_with("import ") {
        return None;
    }
    let open = trimmed.find('{')?;
    let close = trimmed[open..].find('}')? + open;
    let from_pos = trimmed[close + 1..].find("from")? + close + 1;

    let inside = trimmed[open + 1..close].trim();
    let from_part = trimmed[from_pos + 4..].trim();
    let module = unquote(from_part)?;

    let mut specs = Vec::new();
    for chunk in inside.split(',') {
        let part = chunk.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(as_pos) = part.find(" as ") {
            let imported = part[..as_pos].trim();
            let local = part[as_pos + 4..].trim();
            if !imported.is_empty() && !local.is_empty() {
                specs.push(ImportSpec {
                    imported: imported.to_string(),
                    local: local.to_string(),
                });
            }
        } else {
            specs.push(ImportSpec {
                imported: part.to_string(),
                local: part.to_string(),
            });
        }
    }

    if specs.is_empty() {
        return None;
    }

    Some(ImportDecl {
        from: module.to_string(),
        specs,
    })
}

fn parse_export_function_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("export function ") {
        return None;
    }
    let rest = &trimmed[16..];
    let name = rest.split('(').next()?.trim();
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn parse_export_specs_line(line: &str) -> Option<Vec<ImportSpec>> {
    let trimmed = line.trim_end_matches(';').trim();
    if !trimmed.starts_with("export {") || !trimmed.ends_with('}') {
        return None;
    }
    let inner = &trimmed[8..trimmed.len() - 1];
    let mut specs = Vec::new();
    for chunk in inner.split(',') {
        let part = chunk.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(as_pos) = part.find(" as ") {
            let local = part[..as_pos].trim();
            let exported = part[as_pos + 4..].trim();
            if !local.is_empty() && !exported.is_empty() {
                specs.push(ImportSpec {
                    imported: exported.to_string(),
                    local: local.to_string(),
                });
            }
        } else {
            specs.push(ImportSpec {
                imported: part.to_string(),
                local: part.to_string(),
            });
        }
    }
    Some(specs)
}

fn unquote(input: &str) -> Option<&str> {
    let s = input.trim();
    if s.len() < 2 {
        return None;
    }
    let first = s.as_bytes()[0] as char;
    let last = s.as_bytes()[s.len() - 1] as char;
    if (first == '\'' && last == '\'') || (first == '"' && last == '"') {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}

struct JsSubsetEmitter<'a> {
    source: &'a [u8],
    body: String,
    uses_jsx_runtime: bool,
    uses_include_stub: bool,
    scopes: Vec<HashSet<String>>,
    meta: SourceModuleMeta,
    struct_schemas: Vec<(String, String)>,
}

impl<'a> JsSubsetEmitter<'a> {
    fn new(source: &'a [u8], meta: SourceModuleMeta) -> Self {
        Self {
            source,
            body: String::new(),
            uses_jsx_runtime: false,
            uses_include_stub: false,
            scopes: vec![HashSet::new()],
            meta,
            struct_schemas: Vec::new(),
        }
    }

    fn finish(self) -> String {
        let mut out = String::new();
        out.push_str("// Generated by deka build. Do not edit manually.\n");
        out.push_str("// Target semantics: JavaScript runtime semantics.\n");
        out.push_str("export const phpxBuildMode = \"subset-ast\";\n");
        out.push_str("export const phpxTargetSemantics = \"js\";\n\n");

        let mut imports = self.meta.imports.clone();
        if self.uses_jsx_runtime {
            add_or_merge_import(
                &mut imports,
                "component/core",
                vec![
                    ImportSpec {
                        imported: "jsx".to_string(),
                        local: "jsx".to_string(),
                    },
                    ImportSpec {
                        imported: "jsxs".to_string(),
                        local: "jsxs".to_string(),
                    },
                ],
            );
        }
        let deka_i_locals = extract_deka_i_imports(&mut imports);

        for decl in &imports {
            out.push_str("import { ");
            for (idx, spec) in decl.specs.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                if spec.imported == spec.local {
                    out.push_str(&spec.local);
                } else {
                    out.push_str(&format!("{} as {}", spec.imported, spec.local));
                }
            }
            out.push_str(&format!(" }} from '{}';\n", decl.from));
        }

        if !imports.is_empty() {
            out.push('\n');
        }

        if !deka_i_locals.is_empty() {
            out.push_str("const __phpxTypeRegistry = {};\n\n");
            for (name, schema) in &self.struct_schemas {
                out.push_str(&format!("__phpxTypeRegistry[{}] = {};\n", json_string(name), schema));
            }
            out.push('\n');
            out.push_str(&emit_deka_i_runtime());
            for local in &deka_i_locals {
                out.push_str(&format!("const {} = __deka_i;\n", local));
            }
            out.push('\n');
        }

        if self.uses_include_stub {
            out.push_str("function __phpx_include(path, kind) {\n");
            out.push_str("  throw new Error(`include/require not supported in JS subset emitter: ${kind} ${path}`);\n");
            out.push_str("}\n\n");
        }

        out.push_str(&self.body);

        if !self.meta.export_specs.is_empty() {
            out.push('\n');
            out.push_str("export { ");
            for (idx, spec) in self.meta.export_specs.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                if spec.imported == spec.local {
                    out.push_str(&spec.local);
                } else {
                    out.push_str(&format!("{} as {}", spec.local, spec.imported));
                }
            }
            out.push_str(" };\n");
        }

        out
    }

    fn emit_program(&mut self, program: &Program<'_>) -> Result<(), String> {
        for stmt in program.statements {
            self.emit_stmt(*stmt)?;
        }
        Ok(())
    }

    fn emit_stmt(&mut self, stmt: StmtId<'_>) -> Result<(), String> {
        match stmt {
            Stmt::Namespace { .. } => {
                Err("namespace declarations are not supported in JS subset emitter".to_string())
            }
            Stmt::Use { .. } => {
                Err("use declarations are not supported in JS subset emitter".to_string())
            }
            Stmt::Class {
                kind: ClassKind::Struct,
                name,
                members,
                ..
            } => {
                let schema = self.emit_struct_schema(*members);
                self.struct_schemas.push((self.token_name(name), schema));
                Ok(())
            }
            Stmt::Class { .. }
            | Stmt::Trait { .. }
            | Stmt::Interface { .. }
            | Stmt::Enum { .. } => Err(
                "class-like declarations are not supported in JS subset emitter".to_string(),
            ),
            Stmt::TypeAlias { .. } => {
                Err("type aliases are not supported in JS subset emitter".to_string())
            }
            Stmt::Error { .. } => {
                Err("parser error statement reached JS subset emitter".to_string())
            }
            Stmt::Function {
                name, params, body, ..
            } => {
                let fn_name = self.token_name(name);
                let js_params = params
                    .iter()
                    .map(|p| self.token_name(p.name))
                    .collect::<Vec<_>>()
                    .join(", ");

                let exported = self.scopes.len() == 1 && self.meta.exported_functions.contains(&fn_name);
                if exported {
                    self.body
                        .push_str(&format!("export function {}({}) {{\n", fn_name, js_params));
                } else {
                    self.body
                        .push_str(&format!("function {}({}) {{\n", fn_name, js_params));
                }

                self.push_scope();
                for p in *params {
                    self.declare_in_scope(&self.token_name(p.name));
                }
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();

                self.body.push_str("}\n\n");
                Ok(())
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond = self.emit_expr(*condition)?;
                self.body.push_str(&format!("if ({}) {{\n", cond));
                self.push_scope();
                for inner in *then_block {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                self.body.push('}');

                if let Some(else_stmts) = else_block {
                    self.body.push_str(" else {\n");
                    self.push_scope();
                    for inner in *else_stmts {
                        self.emit_stmt(*inner)?;
                    }
                    self.pop_scope();
                    self.body.push('}');
                }
                self.body.push('\n');
                Ok(())
            }
            Stmt::Block { statements, .. } => {
                self.body.push_str("{\n");
                self.push_scope();
                for inner in *statements {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                self.body.push_str("}\n");
                Ok(())
            }
            Stmt::While { condition, body, .. } => {
                let cond = self.emit_expr(*condition)?;
                self.body.push_str(&format!("while ({}) {{\n", cond));
                self.push_scope();
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                self.body.push_str("}\n");
                Ok(())
            }
            Stmt::For {
                init,
                condition,
                loop_expr,
                body,
                ..
            } => {
                self.push_scope();
                let init_js = self.emit_for_init(init)?;
                let cond_js = self.emit_expr_list(condition)?;
                let loop_js = self.emit_expr_list(loop_expr)?;
                self.body
                    .push_str(&format!("for ({}; {}; {}) {{\n", init_js, cond_js, loop_js));
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.body.push_str("}\n");
                self.pop_scope();
                Ok(())
            }
            Stmt::Foreach {
                expr,
                key_var,
                value_var,
                body,
                ..
            } => {
                let iterable = self.emit_expr(*expr)?;
                let value_name = self.extract_var_name(*value_var)?;
                if let Some(key_var) = key_var {
                    let key_name = self.extract_var_name(*key_var)?;
                    self.body.push_str(&format!(
                        "for (const [{} , {}] of Object.entries({})) {{\n",
                        key_name, value_name, iterable
                    ));
                    self.push_scope();
                    self.declare_in_scope(&key_name);
                    self.declare_in_scope(&value_name);
                } else {
                    self.body
                        .push_str(&format!("for (const {} of {}) {{\n", value_name, iterable));
                    self.push_scope();
                    self.declare_in_scope(&value_name);
                }
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                self.body.push_str("}\n");
                Ok(())
            }
            Stmt::DoWhile { body, condition, .. } => {
                self.body.push_str("do {\n");
                self.push_scope();
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                let cond = self.emit_expr(*condition)?;
                self.body.push_str(&format!("}} while ({});\n", cond));
                Ok(())
            }
            Stmt::Switch {
                condition, cases, ..
            } => {
                let cond = self.emit_expr(*condition)?;
                self.body.push_str(&format!("switch ({}) {{\n", cond));
                for case in *cases {
                    if let Some(case_cond) = case.condition {
                        let case_expr = self.emit_expr(case_cond)?;
                        self.body.push_str(&format!("case {}:\n", case_expr));
                    } else {
                        self.body.push_str("default:\n");
                    }
                    self.push_scope();
                    for inner in case.body {
                        self.emit_stmt(*inner)?;
                    }
                    self.pop_scope();
                }
                self.body.push_str("}\n");
                Ok(())
            }
            Stmt::Const { consts, .. } => {
                for item in *consts {
                    let name = self.token_name(item.name);
                    let value = self.emit_expr(item.value)?;
                    self.body.push_str(&format!("const {} = {};\n", name, value));
                    self.declare_in_scope(&name);
                }
                Ok(())
            }
            Stmt::Global { vars, .. } => {
                if !vars.is_empty() {
                    self.body.push_str("// global declarations are no-ops in JS subset mode\n");
                }
                Ok(())
            }
            Stmt::Static { vars, .. } => {
                for item in *vars {
                    let mut init_expr = item.default;
                    let target_expr = match *item.var {
                        Expr::Assign { var, expr, .. } => {
                            if init_expr.is_none() {
                                init_expr = Some(expr);
                            }
                            var
                        }
                        _ => item.var,
                    };

                    if let Some(raw_name) = self.extract_static_var_name(target_expr) {
                        let var_name =
                            self.sanitize_name(raw_name.split('=').next().unwrap_or(raw_name.as_str()));
                        let init = if let Some(default) = init_expr {
                            self.emit_expr(default)?
                        } else {
                            "undefined".to_string()
                        };
                        if !self.is_declared(&var_name) {
                            self.body.push_str(&format!("let {} = {};\n", var_name, init));
                            self.declare_in_scope(&var_name);
                        } else {
                            self.body.push_str(&format!("{} = {};\n", var_name, init));
                        }
                    } else {
                        self.body.push_str(
                            "// unsupported static declaration target in JS subset mode\n",
                        );
                    }
                }
                Ok(())
            }
            Stmt::Unset { vars, .. } => {
                for var in *vars {
                    let target = self.emit_expr(*var)?;
                    self.body.push_str(&format!("{} = {};\n", target, "undefined"));
                }
                Ok(())
            }
            Stmt::Label { name, .. } => {
                self.body.push_str(&format!(
                    "// label {} ignored in JS subset mode\n",
                    self.token_name(name)
                ));
                Ok(())
            }
            Stmt::Goto { label, .. } => {
                self.body.push_str(&format!(
                    "// goto {} is not supported in JS subset mode\n",
                    self.token_name(label)
                ));
                Ok(())
            }
            Stmt::Declare { body, .. } => {
                // PHP declare directives have no JS equivalent; emit body directly.
                self.push_scope();
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                Ok(())
            }
            Stmt::HaltCompiler { .. } => {
                self.body
                    .push_str("// __halt_compiler ignored in JS subset mode\n");
                Ok(())
            }
            Stmt::Break { .. } => {
                self.body.push_str("break;\n");
                Ok(())
            }
            Stmt::Continue { .. } => {
                self.body.push_str("continue;\n");
                Ok(())
            }
            Stmt::Return { expr, .. } => {
                if let Some(expr) = expr {
                    let value = self.emit_expr(*expr)?;
                    self.body.push_str(&format!("return {};\n", value));
                } else {
                    self.body.push_str("return;\n");
                }
                Ok(())
            }
            Stmt::Throw { expr, .. } => {
                let value = self.emit_expr(*expr)?;
                self.body.push_str(&format!("throw {};\n", value));
                Ok(())
            }
            Stmt::Try {
                body,
                catches,
                finally,
                ..
            } => {
                self.body.push_str("try {\n");
                self.push_scope();
                for inner in *body {
                    self.emit_stmt(*inner)?;
                }
                self.pop_scope();
                self.body.push_str("}");

                if let Some(first_catch) = catches.first() {
                    let err_name = if let Some(var) = first_catch.var {
                        let name = self.token_name(var);
                        if name.is_empty() { "err".to_string() } else { name }
                    } else {
                        "err".to_string()
                    };
                    self.body.push_str(&format!(" catch ({}) {{\n", err_name));
                    self.push_scope();
                    self.declare_in_scope(&err_name);
                    for inner in first_catch.body {
                        self.emit_stmt(*inner)?;
                    }
                    self.pop_scope();
                    self.body.push_str("}");
                }

                if let Some(finally_block) = finally {
                    self.body.push_str(" finally {\n");
                    self.push_scope();
                    for inner in *finally_block {
                        self.emit_stmt(*inner)?;
                    }
                    self.pop_scope();
                    self.body.push_str("}");
                }

                self.body.push('\n');
                Ok(())
            }
            Stmt::Echo { exprs, .. } => {
                for expr in *exprs {
                    let value = self.emit_expr(*expr)?;
                    self.body.push_str(&format!("console.log({});\n", value));
                }
                Ok(())
            }
            Stmt::Expression { expr, .. } => {
                if let Some((name, rhs)) = self.assignment_to_named_var(*expr)? {
                    if !self.is_declared(&name) {
                        self.declare_in_scope(&name);
                        self.body.push_str(&format!("let {} = {};\n", name, rhs));
                    } else {
                        self.body.push_str(&format!("{} = {};\n", name, rhs));
                    }
                } else {
                    let value = self.emit_expr(*expr)?;
                    self.body.push_str(&format!("{};\n", value));
                }
                Ok(())
            }
            Stmt::InlineHtml { value, .. } => {
                let text = String::from_utf8_lossy(value);
                self.body.push_str(&format!("// inline html: {}\n", text.replace('\n', "\\n")));
                Ok(())
            }
            Stmt::Nop { .. } => Ok(()),
        }
    }

    fn emit_expr(&mut self, expr: ExprId<'_>) -> Result<String, String> {
        match expr {
            Expr::Variable { name, .. } => Ok(self.span_name(*name)),
            Expr::Integer { value, .. } | Expr::Float { value, .. } => {
                Ok(String::from_utf8_lossy(value).to_string())
            }
            Expr::Boolean { value, .. } => Ok(if *value { "true" } else { "false" }.to_string()),
            Expr::Null { .. } => Ok("null".to_string()),
            Expr::String { value, .. } => Ok(self.encode_php_string_literal(value)),
            Expr::Unary { op, expr, .. } => {
                let value = self.emit_expr(*expr)?;
                let js_op = match op {
                    UnaryOp::Plus => "+",
                    UnaryOp::Minus => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::BitNot => "~",
                    UnaryOp::PreInc => "++",
                    UnaryOp::PreDec => "--",
                    _ => {
                        return Err(format!("unsupported unary operator in subset emitter: {:?}", op));
                    }
                };
                Ok(format!("({}{})", js_op, value))
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                let lhs = self.emit_expr(*left)?;
                let rhs = self.emit_expr(*right)?;
                let js_op = match op {
                    BinaryOp::Plus => "+",
                    BinaryOp::Minus => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Pow => "**",
                    BinaryOp::Concat => "+",
                    BinaryOp::Eq | BinaryOp::EqEq => "===",
                    BinaryOp::EqEqEq => "===",
                    BinaryOp::NotEq => "!==",
                    BinaryOp::NotEqEq => "!==",
                    BinaryOp::Lt => "<",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::And | BinaryOp::LogicalAnd => "&&",
                    BinaryOp::Or | BinaryOp::LogicalOr => "||",
                    BinaryOp::Coalesce => "??",
                    _ => {
                        return Err(format!("unsupported binary operator in subset emitter: {:?}", op));
                    }
                };
                Ok(format!("({} {} {})", lhs, js_op, rhs))
            }
            Expr::ArrowFunction {
                params, expr, ..
            } => {
                let mut names = Vec::with_capacity(params.len());
                for param in *params {
                    names.push(self.token_name(param.name));
                }
                let body = self.emit_expr(*expr)?;
                Ok(format!("({}) => {}", names.join(", "), body))
            }
            Expr::Closure { params, body, .. } => {
                let mut names = Vec::with_capacity(params.len());
                for param in *params {
                    names.push(self.token_name(param.name));
                }
                let block = self.emit_stmt_block_inline(body)?;
                Ok(format!("function({}) {{\n{} }}", names.join(", "), block))
            }
            Expr::Call { func, args, .. } => {
                let callee = self.emit_expr(*func)?;
                let args_js = self.emit_call_args(args)?;
                Ok(format!("{}({})", callee, args_js))
            }
            Expr::Assign { var, expr, .. } => {
                if let Expr::Variable { name, .. } = *var {
                    let js_name = self.span_name(*name);
                    let rhs = self.emit_expr(*expr)?;
                    Ok(format!("({} = {})", js_name, rhs))
                } else {
                    Err("subset emitter only supports assignment to simple variables".to_string())
                }
            }
            Expr::AssignRef { var, expr, .. } => {
                if let Expr::Variable { name, .. } = *var {
                    let js_name = self.span_name(*name);
                    let rhs = self.emit_expr(*expr)?;
                    Ok(format!("({} = {})", js_name, rhs))
                } else {
                    Err("subset emitter only supports assignment to simple variables".to_string())
                }
            }
            Expr::AssignOp { var, op, expr, .. } => {
                if let Expr::Variable { name, .. } = *var {
                    let js_name = self.span_name(*name);
                    let rhs = self.emit_expr(*expr)?;
                    let js_op = match op {
                        php_rs::parser::ast::AssignOp::Plus => "+=",
                        php_rs::parser::ast::AssignOp::Minus => "-=",
                        php_rs::parser::ast::AssignOp::Mul => "*=",
                        php_rs::parser::ast::AssignOp::Div => "/=",
                        php_rs::parser::ast::AssignOp::Mod => "%=",
                        php_rs::parser::ast::AssignOp::Concat => "+=",
                        php_rs::parser::ast::AssignOp::BitAnd => "&=",
                        php_rs::parser::ast::AssignOp::BitOr => "|=",
                        php_rs::parser::ast::AssignOp::BitXor => "^=",
                        php_rs::parser::ast::AssignOp::ShiftLeft => "<<=",
                        php_rs::parser::ast::AssignOp::ShiftRight => ">>=",
                        php_rs::parser::ast::AssignOp::Pow => "**=",
                        php_rs::parser::ast::AssignOp::Coalesce => "??=",
                    };
                    Ok(format!("({} {} {})", js_name, js_op, rhs))
                } else {
                    Err("subset emitter only supports assignment to simple variables".to_string())
                }
            }
            Expr::PropertyFetch {
                target, property, ..
            } => {
                let target_js = self.emit_expr(*target)?;
                match *property {
                    Expr::Variable { name, .. } => {
                        let prop = self.span_name(*name);
                        Ok(format!("{}.{}", target_js, prop))
                    }
                    _ => Err("dynamic property access is not supported in subset emitter".to_string()),
                }
            }
            Expr::MethodCall {
                target,
                method,
                args,
                ..
            } => {
                let target_js = self.emit_expr(*target)?;
                let method_name = match *method {
                    Expr::Variable { name, .. } => self.span_name(*name),
                    _ => {
                        return Err(
                            "dynamic method calls are not supported in subset emitter".to_string(),
                        )
                    }
                };
                let args_js = self.emit_call_args(args)?;
                Ok(format!("{}.{}({})", target_js, method_name, args_js))
            }
            Expr::StaticCall {
                class,
                method,
                args,
                ..
            } => {
                let class_js = self.emit_expr(*class)?;
                let method_name = match *method {
                    Expr::Variable { name, .. } => self.span_name(*name),
                    _ => {
                        return Err(
                            "dynamic static method calls are not supported in subset emitter"
                                .to_string(),
                        )
                    }
                };
                let args_js = self.emit_call_args(args)?;
                Ok(format!("{}.{}({})", class_js, method_name, args_js))
            }
            Expr::ClassConstFetch {
                class, constant, ..
            } => {
                let class_js = self.emit_expr(*class)?;
                let const_name = match *constant {
                    Expr::Variable { name, .. } => self.span_name(*name),
                    _ => {
                        return Err(
                            "dynamic class constant access is not supported in subset emitter"
                                .to_string(),
                        )
                    }
                };
                Ok(format!("{}.{}", class_js, const_name))
            }
            Expr::New { class, args, .. } => {
                let class_js = self.emit_expr(*class)?;
                let args_js = self.emit_call_args(args)?;
                Ok(format!("new {}({})", class_js, args_js))
            }
            Expr::NullsafePropertyFetch {
                target, property, ..
            } => {
                let target_js = self.emit_expr(*target)?;
                match *property {
                    Expr::Variable { name, .. } => {
                        let prop = self.span_name(*name);
                        Ok(format!("({})?.{}", target_js, prop))
                    }
                    _ => Err(
                        "dynamic nullsafe property access is not supported in subset emitter"
                            .to_string(),
                    ),
                }
            }
            Expr::NullsafeMethodCall {
                target,
                method,
                args,
                ..
            } => {
                let target_js = self.emit_expr(*target)?;
                let method_name = match *method {
                    Expr::Variable { name, .. } => self.span_name(*name),
                    _ => {
                        return Err(
                            "dynamic nullsafe method calls are not supported in subset emitter"
                                .to_string(),
                        )
                    }
                };
                let args_js = self.emit_call_args(args)?;
                Ok(format!("({})?.{}({})", target_js, method_name, args_js))
            }
            Expr::DotAccess {
                target, property, ..
            } => {
                let target_js = self.emit_expr(*target)?;
                let prop = self.token_text(property);
                Ok(format!("{}.{}", target_js, prop))
            }
            Expr::ArrayDimFetch { array, dim, .. } => {
                let array_js = self.emit_expr(*array)?;
                if let Some(dim) = dim {
                    let dim_js = self.emit_expr(*dim)?;
                    Ok(format!("{}[{}]", array_js, dim_js))
                } else {
                    Err("append array access is not supported in subset emitter".to_string())
                }
            }
            Expr::Array { items, .. } => {
                if items.iter().all(|item| item.key.is_none() && !item.by_ref && !item.unpack) {
                    let mut values = Vec::new();
                    for item in *items {
                        values.push(self.emit_expr(item.value)?);
                    }
                    return Ok(format!("[{}]", values.join(", ")));
                }

                if items.iter().all(|item| item.key.is_some() && !item.by_ref && !item.unpack) {
                    let mut entries = Vec::new();
                    for item in *items {
                        let key_expr = item
                            .key
                            .ok_or_else(|| "keyed array expected key".to_string())?;
                        let key = self.emit_static_array_key(key_expr)?;
                        let value = self.emit_expr(item.value)?;
                        entries.push(format!("{}: {}", json_string(&key), value));
                    }
                    return Ok(format!("{{{}}}", entries.join(", ")));
                }

                Err("mixed or complex array items are not supported in subset emitter".to_string())
            }
            Expr::ObjectLiteral { items, .. } => {
                let mut entries = Vec::new();
                for item in *items {
                    let key = match item.key {
                        ObjectKey::Ident(tok) | ObjectKey::String(tok) => self.token_text(tok),
                    };
                    let value = self.emit_expr(item.value)?;
                    entries.push(format!("{}: {}", json_string(&key), value));
                }
                Ok(format!("{{{}}}", entries.join(", ")))
            }
            Expr::StructLiteral { name, fields, .. } => {
                let mut entries = Vec::new();
                entries.push(format!(
                    "{}: {}",
                    json_string("__struct"),
                    json_string(&self.name_last_segment(*name))
                ));
                for field in *fields {
                    let key = self.token_text(field.name).trim_start_matches('$').to_string();
                    let value = self.emit_expr(field.value)?;
                    entries.push(format!("{}: {}", json_string(&key), value));
                }
                Ok(format!("{{{}}}", entries.join(", ")))
            }
            Expr::JsxElement {
                name,
                attributes,
                children,
                ..
            } => self.emit_jsx(Some(*name), attributes, children),
            Expr::JsxFragment { children, .. } => self.emit_jsx(None, &[], children),
            Expr::Cast { kind, expr, .. } => {
                let value = self.emit_expr(*expr)?;
                let lowered = match kind {
                    php_rs::parser::ast::CastKind::Int => format!("Number.parseInt({}, 10)", value),
                    php_rs::parser::ast::CastKind::Float => format!("Number({})", value),
                    php_rs::parser::ast::CastKind::String => format!("String({})", value),
                    php_rs::parser::ast::CastKind::Bool => format!("Boolean({})", value),
                    php_rs::parser::ast::CastKind::Array => format!("Array.isArray({0}) ? {0} : [{0}]", value),
                    php_rs::parser::ast::CastKind::Object => format!("({})", value),
                    _ => return Err(format!("unsupported cast kind in subset emitter: {:?}", kind)),
                };
                Ok(lowered)
            }
            Expr::Isset { vars, .. } => {
                if vars.is_empty() {
                    return Ok("false".to_string());
                }
                let mut checks = Vec::with_capacity(vars.len());
                for var in *vars {
                    let value = self.emit_expr(*var)?;
                    checks.push(format!("({0} !== undefined && {0} !== null)", value));
                }
                Ok(format!("({})", checks.join(" && ")))
            }
            Expr::PostInc { var, .. } => {
                if let Expr::Variable { name, .. } = *var {
                    Ok(format!("({}++)", self.span_name(*name)))
                } else {
                    Err("subset emitter only supports ++ on simple variables".to_string())
                }
            }
            Expr::PostDec { var, .. } => {
                if let Expr::Variable { name, .. } = *var {
                    Ok(format!("({}--)", self.span_name(*name)))
                } else {
                    Err("subset emitter only supports -- on simple variables".to_string())
                }
            }
            Expr::Empty { expr, .. } => {
                let value = self.emit_expr(*expr)?;
                Ok(format!("(!({}))", value))
            }
            Expr::Print { expr, .. } => {
                let value = self.emit_expr(*expr)?;
                Ok(format!("(console.log({}), undefined)", value))
            }
            Expr::Await { expr, .. } => {
                let value = self.emit_expr(*expr)?;
                Ok(format!("(await {})", value))
            }
            Expr::Eval { expr, .. } => {
                let value = self.emit_expr(*expr)?;
                Ok(format!("eval({})", value))
            }
            Expr::Clone { expr, .. } => {
                let value = self.emit_expr(*expr)?;
                Ok(format!("structuredClone({})", value))
            }
            Expr::Die { expr, .. } | Expr::Exit { expr, .. } => {
                if let Some(value) = expr {
                    let rendered = self.emit_expr(*value)?;
                    Ok(format!(
                        "(() => {{ throw new Error(String({})); }})()",
                        rendered
                    ))
                } else {
                    Ok("(() => { throw new Error(\"exit\"); })()".to_string())
                }
            }
            Expr::ShellExec { .. } => Err(
                "shell execution is not supported in JS subset emitter".to_string(),
            ),
            Expr::Yield { .. } => {
                Err("yield expressions are not supported in JS subset emitter".to_string())
            }
            Expr::AnonymousClass { .. } => Err(
                "anonymous classes are not supported in JS subset emitter".to_string(),
            ),
            Expr::VariadicPlaceholder { .. } => Err(
                "variadic placeholder is not supported in JS subset emitter".to_string(),
            ),
            Expr::Error { .. } => {
                Err("parser error expression reached JS subset emitter".to_string())
            }
            Expr::Include { kind, expr, .. } => {
                self.uses_include_stub = true;
                let path = self.emit_expr(*expr)?;
                let kind_name = match kind {
                    php_rs::parser::ast::IncludeKind::Include => "include",
                    php_rs::parser::ast::IncludeKind::IncludeOnce => "include_once",
                    php_rs::parser::ast::IncludeKind::Require => "require",
                    php_rs::parser::ast::IncludeKind::RequireOnce => "require_once",
                };
                Ok(format!("__phpx_include({}, {})", path, json_string(kind_name)))
            }
            Expr::MagicConst { kind, .. } => {
                let lowered = match kind {
                    php_rs::parser::ast::MagicConstKind::Line => "0".to_string(),
                    php_rs::parser::ast::MagicConstKind::Dir
                    | php_rs::parser::ast::MagicConstKind::File
                    | php_rs::parser::ast::MagicConstKind::Function
                    | php_rs::parser::ast::MagicConstKind::Class
                    | php_rs::parser::ast::MagicConstKind::Trait
                    | php_rs::parser::ast::MagicConstKind::Method
                    | php_rs::parser::ast::MagicConstKind::Namespace
                    | php_rs::parser::ast::MagicConstKind::Property => json_string(""),
                };
                Ok(lowered)
            }
            Expr::InterpolatedString { parts, .. } => {
                let mut pieces = Vec::new();
                for part in *parts {
                    pieces.push(self.emit_expr(*part)?);
                }
                Ok(format!("({})", pieces.join(" + ")))
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                let cond = self.emit_expr(*condition)?;
                let when_true = if let Some(value) = if_true {
                    self.emit_expr(*value)?
                } else {
                    cond.clone()
                };
                let when_false = self.emit_expr(*if_false)?;
                Ok(format!("({} ? {} : {})", cond, when_true, when_false))
            }
            Expr::Match {
                condition, arms, ..
            } => self.emit_match_expr(*condition, arms),
            other => Err(format!("unsupported expression in subset emitter: {:?}", other)),
        }
    }

    fn emit_jsx(
        &mut self,
        name: Option<php_rs::parser::ast::Name<'_>>,
        attributes: &[php_rs::parser::ast::JsxAttribute<'_>],
        children: &[JsxChild<'_>],
    ) -> Result<String, String> {
        self.uses_jsx_runtime = true;

        let mut props = Vec::new();
        for attr in attributes {
            let key = self.token_text(attr.name);
            let value = if let Some(expr) = attr.value {
                self.emit_expr(expr)?
            } else {
                "true".to_string()
            };
            props.push(format!("{}: {}", json_string(&key), value));
        }

        let mut child_values = Vec::new();
        for child in children {
            match child {
                JsxChild::Text(span) => {
                    if let Some(text) = self.normalize_jsx_text(*span) {
                        child_values.push(json_string(&text));
                    }
                }
                JsxChild::Expr(expr) => child_values.push(self.emit_expr(*expr)?),
            }
        }

        if !child_values.is_empty() {
            if child_values.len() == 1 {
                props.push(format!("\"children\": {}", child_values[0]));
            } else {
                props.push(format!("\"children\": [{}]", child_values.join(", ")));
            }
        }

        let tag_expr = match name {
            Some(n) => {
                let raw = self.span_bytes(n.span);
                let trimmed = raw.strip_prefix(b"\\").unwrap_or(raw);
                let raw_text = String::from_utf8_lossy(trimmed).to_string();
                let last = raw_text.rsplit('\\').next().unwrap_or(raw_text.as_str());
                let is_component = last
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_uppercase())
                    .unwrap_or(false);
                if is_component {
                    last.to_string()
                } else {
                    json_string(last)
                }
            }
            None => json_string("__fragment__"),
        };

        let props_expr = format!("{{{}}}", props.join(", "));
        let fn_name = if child_values.len() > 1 { "jsxs" } else { "jsx" };

        Ok(format!("{}({}, {})", fn_name, tag_expr, props_expr))
    }

    fn emit_stmt_block_inline(&mut self, stmts: &[StmtId<'_>]) -> Result<String, String> {
        let saved = std::mem::take(&mut self.body);
        self.push_scope();
        for stmt in stmts {
            self.emit_stmt(*stmt)?;
        }
        self.pop_scope();
        let block = std::mem::take(&mut self.body);
        self.body = saved;
        Ok(block)
    }

    fn emit_expr_list(&mut self, exprs: &[ExprId<'_>]) -> Result<String, String> {
        if exprs.is_empty() {
            return Ok(String::new());
        }
        let mut out = Vec::with_capacity(exprs.len());
        for expr in exprs {
            out.push(self.emit_expr(*expr)?);
        }
        Ok(out.join(", "))
    }

    fn emit_for_init(&mut self, exprs: &[ExprId<'_>]) -> Result<String, String> {
        if exprs.is_empty() {
            return Ok(String::new());
        }

        let mut parts = Vec::with_capacity(exprs.len());
        let mut all_new_assignments = true;

        for expr in exprs {
            if let Some((name, rhs)) = self.assignment_to_named_var(*expr)? {
                if self.is_declared(&name) {
                    all_new_assignments = false;
                    parts.push(format!("{} = {}", name, rhs));
                } else {
                    self.declare_in_scope(&name);
                    parts.push(format!("{} = {}", name, rhs));
                }
            } else {
                all_new_assignments = false;
                parts.push(self.emit_expr(*expr)?);
            }
        }

        if all_new_assignments {
            Ok(format!("let {}", parts.join(", ")))
        } else {
            Ok(parts.join(", "))
        }
    }

    fn extract_var_name(&self, expr: ExprId<'_>) -> Result<String, String> {
        match expr {
            Expr::Variable { name, .. } => Ok(self.span_name(*name)),
            _ => Err("foreach key/value target must be a variable in subset emitter".to_string()),
        }
    }

    fn extract_static_var_name(&self, expr: ExprId<'_>) -> Option<String> {
        match expr {
            Expr::Variable { name, .. } => Some(self.span_name(*name)),
            Expr::IndirectVariable { name, .. } => match *name {
                Expr::Variable { name, .. } => Some(self.span_name(*name)),
                _ => Some(self.span_name(name.span())),
            },
            _ => Some(self.span_name(expr.span())),
        }
    }

    fn emit_call_args(&mut self, args: &[php_rs::parser::ast::Arg<'_>]) -> Result<String, String> {
        if args.iter().all(|a| a.name.is_none() && !a.unpack) {
            let mut rendered = Vec::with_capacity(args.len());
            for arg in args {
                rendered.push(self.emit_expr(arg.value)?);
            }
            return Ok(rendered.join(", "));
        }

        if args.iter().all(|a| a.name.is_some() && !a.unpack) {
            let mut entries = Vec::with_capacity(args.len());
            for arg in args {
                let name = arg
                    .name
                    .map(|tok| self.sanitize_name(&self.token_text(tok)))
                    .ok_or_else(|| {
                        "mixed positional/named arguments are not supported in subset emitter"
                            .to_string()
                    })?;
                let value = self.emit_expr(arg.value)?;
                entries.push(format!("{}: {}", json_string(&name), value));
            }
            return Ok(format!("{{{}}}", entries.join(", ")));
        }

        Err("mixed positional/named/unpack call arguments are not supported in subset emitter"
            .to_string())
    }

    fn emit_match_expr(
        &mut self,
        condition: ExprId<'_>,
        arms: &[php_rs::parser::ast::MatchArm<'_>],
    ) -> Result<String, String> {
        let condition_js = self.emit_expr(condition)?;
        let mut rendered = String::new();

        for arm in arms.iter().rev() {
            let arm_expr = self.emit_expr(arm.body)?;
            if arm.conditions.is_none() {
                rendered = arm_expr;
                continue;
            }
            let guard = self.emit_match_guard(&condition_js, arm.conditions)?;
            if rendered.is_empty() {
                rendered = format!("({} ? {} : undefined)", guard, arm_expr);
            } else {
                rendered = format!("({} ? {} : {})", guard, arm_expr, rendered);
            }
        }

        if rendered.is_empty() {
            return Err("match requires at least one arm".to_string());
        }
        Ok(rendered)
    }

    fn emit_match_guard(
        &mut self,
        condition_js: &str,
        conditions: Option<&[ExprId<'_>]>,
    ) -> Result<String, String> {
        let Some(conditions) = conditions else {
            return Ok("true".to_string());
        };
        if conditions.is_empty() {
            return Ok("false".to_string());
        }

        let mut checks = Vec::with_capacity(conditions.len());
        for cond in conditions {
            let rhs = self.emit_expr(*cond)?;
            checks.push(format!("({} === {})", condition_js, rhs));
        }
        Ok(format!("({})", checks.join(" || ")))
    }

    fn emit_static_array_key(&self, expr: ExprId<'_>) -> Result<String, String> {
        match expr {
            Expr::String { value, .. } => {
                let mut bytes: &[u8] = value;
                if bytes.len() >= 2
                    && ((bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
                        || (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'))
                {
                    bytes = &bytes[1..bytes.len() - 1];
                }
                Ok(String::from_utf8_lossy(bytes).to_string())
            }
            Expr::Integer { value, .. } => Ok(String::from_utf8_lossy(value).to_string()),
            Expr::Variable { name, .. } => Ok(self.span_name(*name)),
            _ => Err("array key must be static string/int/identifier in subset emitter".to_string()),
        }
    }

    fn assignment_to_named_var(&mut self, expr: ExprId<'_>) -> Result<Option<(String, String)>, String> {
        match expr {
            Expr::Assign { var, expr, .. } | Expr::AssignRef { var, expr, .. } => {
                if let Expr::Variable { name, .. } = *var {
                    let js_name = self.span_name(*name);
                    let rhs = self.emit_expr(*expr)?;
                    Ok(Some((js_name, rhs)))
                } else {
                    Err("subset emitter only supports assignment to simple variables".to_string())
                }
            }
            _ => Ok(None),
        }
    }

    fn name_last_segment(&self, name: php_rs::parser::ast::Name<'_>) -> String {
        if let Some(last) = name.parts.last() {
            self.token_text(last).trim_start_matches('\\').to_string()
        } else {
            let raw = String::from_utf8_lossy(self.span_bytes(name.span)).to_string();
            raw.rsplit('\\').next().unwrap_or(raw.as_str()).to_string()
        }
    }

    fn token_name(&self, tok: &php_rs::parser::lexer::token::Token) -> String {
        self.sanitize_name(self.token_text(tok).as_str())
    }

    fn span_name(&self, span: php_rs::parser::span::Span) -> String {
        let text = String::from_utf8_lossy(self.span_bytes(span)).to_string();
        self.sanitize_name(&text)
    }

    fn sanitize_name(&self, raw: &str) -> String {
        let mut name = raw.trim().trim_start_matches('$').replace('\\', "_");
        if name.is_empty() {
            name = "_".to_string();
        }
        name
    }

    fn token_text(&self, tok: &php_rs::parser::lexer::token::Token) -> String {
        String::from_utf8_lossy(tok.text(self.source)).to_string()
    }

    fn span_bytes(&self, span: php_rs::parser::span::Span) -> &'a [u8] {
        &self.source[span.start..span.end]
    }

    fn encode_php_string_literal(&self, value: &[u8]) -> String {
        let mut bytes: &[u8] = value;
        if bytes.len() >= 2
            && ((bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
                || (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'))
        {
            bytes = &bytes[1..bytes.len() - 1];
        }
        json_string(&String::from_utf8_lossy(bytes))
    }

    fn normalize_jsx_text(&self, span: php_rs::parser::span::Span) -> Option<String> {
        let raw = String::from_utf8_lossy(self.span_bytes(span)).to_string();
        if raw.chars().all(|c| c.is_whitespace()) {
            if raw.contains('\n') || raw.contains('\r') {
                None
            } else {
                Some(" ".to_string())
            }
        } else {
            Some(raw)
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        let _ = self.scopes.pop();
    }

    fn declare_in_scope(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    fn emit_struct_schema(&self, members: &[ClassMember<'_>]) -> String {
        let mut fields = Vec::new();
        for member in members {
            if let ClassMember::Property { ty, entries, .. } = member {
                for entry in *entries {
                    let (schema, optional) = match ty {
                        Some(ty) => self.emit_type_schema(ty),
                        None => ("{ kind: 'unknown' }".to_string(), false),
                    };
                    let name = self.token_name(entry.name);
                    fields.push(format!(
                        "{}: {{ schema: {}, optional: {} }}",
                        json_string(&name),
                        schema,
                        if optional { "true" } else { "false" }
                    ));
                }
            }
        }
        format!("{{ kind: 'object', fields: {{ {} }} }}", fields.join(", "))
    }

    fn emit_type_schema(&self, ty: &AstType<'_>) -> (String, bool) {
        match ty {
            AstType::Simple(tok) => match self.token_name(tok).as_str() {
                "string" => ("{ kind: 'string' }".to_string(), false),
                "int" | "float" | "number" => ("{ kind: 'number' }".to_string(), false),
                "bool" | "boolean" => ("{ kind: 'boolean' }".to_string(), false),
                _ => ("{ kind: 'unknown' }".to_string(), false),
            },
            AstType::Name(_) => ("{ kind: 'object' }".to_string(), false),
            AstType::Nullable(inner) => {
                let (inner_schema, _) = self.emit_type_schema(inner);
                (format!("{{ kind: 'optional', inner: {} }}", inner_schema), true)
            }
            AstType::Union(parts) => {
                let schemas = parts
                    .iter()
                    .map(|part| self.emit_type_schema(part).0)
                    .collect::<Vec<_>>();
                (format!("{{ kind: 'union', anyOf: [{}] }}", schemas.join(", ")), false)
            }
            AstType::Intersection(_parts) => ("{ kind: 'object' }".to_string(), false),
            AstType::ObjectShape(shape_fields) => {
                let fields = shape_fields
                    .iter()
                    .map(|field| {
                        let (inner, opt) = self.emit_type_schema(field.ty);
                        format!(
                            "{}: {{ schema: {}, optional: {} }}",
                            json_string(&self.token_name(field.name)),
                            inner,
                            if field.optional || opt { "true" } else { "false" }
                        )
                    })
                    .collect::<Vec<_>>();
                (format!("{{ kind: 'object', fields: {{ {} }} }}", fields.join(", ")), false)
            }
            AstType::Applied { base, args } => {
                if let AstType::Simple(tok) = *base {
                    let base_name = self.token_name(tok);
                    if base_name == "Option" {
                        let inner = args
                            .first()
                            .map(|t| self.emit_type_schema(t).0)
                            .unwrap_or_else(|| "{ kind: 'unknown' }".to_string());
                        return (format!("{{ kind: 'optional', inner: {} }}", inner), true);
                    }
                    if base_name == "array" || base_name == "Vec" {
                        let inner = args
                            .first()
                            .map(|t| self.emit_type_schema(t).0)
                            .unwrap_or_else(|| "{ kind: 'unknown' }".to_string());
                        return (format!("{{ kind: 'array', item: {} }}", inner), false);
                    }
                }
                ("{ kind: 'unknown' }".to_string(), false)
            }
        }
    }
}

fn add_or_merge_import(imports: &mut Vec<ImportDecl>, from: &str, specs: Vec<ImportSpec>) {
    if let Some(existing) = imports.iter_mut().find(|decl| decl.from == from) {
        for spec in specs {
            if !existing
                .specs
                .iter()
                .any(|item| item.imported == spec.imported && item.local == spec.local)
            {
                existing.specs.push(spec);
            }
        }
        return;
    }
    imports.push(ImportDecl {
        from: from.to_string(),
        specs,
    });
}

fn extract_deka_i_imports(imports: &mut Vec<ImportDecl>) -> Vec<String> {
    let mut locals = Vec::new();
    let mut kept = Vec::new();
    for decl in imports.drain(..) {
        if decl.from == "deka/i" {
            for spec in decl.specs {
                if !locals.contains(&spec.local) {
                    locals.push(spec.local);
                }
            }
        } else {
            kept.push(decl);
        }
    }
    *imports = kept;
    locals
}

fn emit_deka_i_runtime() -> String {
    include_str!("deka_i_runtime.js").to_string()
}

fn json_string(input: &str) -> String {
    serde_json::to_string(input).unwrap_or_else(|_| "\"\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_rs::parser::lexer::Lexer;
    use php_rs::parser::parser::{Parser, ParserMode};

    #[test]
    fn default_outdir_is_dist_js() {
        let path = resolve_output_path(None, Path::new("src/home.phpx")).expect("path");
        assert_eq!(path, PathBuf::from("dist/home.js"));
    }

    #[test]
    fn explicit_out_is_used() {
        let path =
            resolve_output_path(Some("build/out.js".to_string()), Path::new("src/home.phpx"))
                .expect("path");
        assert_eq!(path, PathBuf::from("build/out.js"));
    }

    #[test]
    fn parses_import_and_export_meta() {
        let source = r#"
---
import { jsx, jsxs as pack } from 'component/core'
export function Card($props: object) {
  return <div>{$props.title}</div>
}
export { Card as Panel }
---
<div>ok</div>
"#;
        let meta = parse_source_module_meta(source);
        assert_eq!(meta.imports.len(), 1);
        assert!(meta.exported_functions.contains("Card"));
        assert_eq!(meta.export_specs.len(), 1);
        assert_eq!(meta.export_specs[0].local, "Card");
        assert_eq!(meta.export_specs[0].imported, "Panel");
    }

    #[test]
    fn emitted_js_contains_runtime_hook_for_fallback() {
        let js = emit_js_scaffold_with_reason("echo 'hi'", "main.phpx", "x");
        assert!(js.contains("runPhpx"));
        assert!(js.contains("runtime.executePhpx"));
        assert!(js.contains("phpxTargetSemantics = \"js\""));
    }

    #[test]
    fn rewrites_deka_i_import_to_runtime_helper() {
        let source = r#"
function Hello($props: object) {
  return <span>Hello {$props.name}</span>
}
$view = <div><Hello name="world" /></div>
"#;
        let meta_source = "---
import { i } from 'deka/i'
---
<div />
";
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let meta = parse_source_module_meta(meta_source);
        let js = emit_js_from_ast(&program, source.as_bytes(), meta).expect("subset emit");
        assert!(!js.contains("from 'deka/i'"));
        assert!(js.contains("const __phpxTypeRegistry = {}"));
        assert!(js.contains("const i = __deka_i;"));
        assert!(js.contains("safeParse"));
    }

    #[test]
    fn captures_struct_schema_for_deka_i_registry() {
        let source = r#"
struct User {
  $name: string
  $age: Option<int>
}
function Hello($props: object) {
  return <span>Hello {$props.name}</span>
}
$view = <div><Hello name="world" /></div>
"#;
        let meta_source = "---
import { i } from 'deka/i'
---
<div />
";
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let meta = parse_source_module_meta(meta_source);
        let js = emit_js_from_ast(&program, source.as_bytes(), meta).expect("subset emit");
        assert!(js.contains("__phpxTypeRegistry[\"User\"]"));
        assert!(js.contains("\"name\": { schema: { kind: 'string' }, optional: false }"));
        assert!(js.contains("\"age\": { schema:"));
        assert!(js.contains("kind: 'number'"));
    }

    #[test]
    fn emits_subset_ast_for_jsx_component() {
        let source = r#"
function Hello($props: object) {
  return <span style="color:red">Hello {$props.name}</span>
}
$view = <div><Hello name="world" /></div>
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("phpxBuildMode = \"subset-ast\""));
        assert!(js.contains("phpxTargetSemantics = \"js\""));
        assert!(js.contains("import { jsx, jsxs } from 'component/core'"));
        assert!(js.contains("function Hello(props)"));
        assert!(js.contains("let view = jsx"));
    }

    #[test]
    fn emits_if_and_unary() {
        let source = r#"
function Flag($props: object) {
  if (!$props.on) {
    return "off"
  }
  return "on"
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("if ((!props.on))"));
        assert!(js.contains("return \"off\""));
    }
    #[test]
    fn emits_match_and_array_dim_fetch() {
        let source = r#"
function Pick($props: object) {
  $name = match ($props.role) {
    "owner" => "sam",
    "member", "user" => "guest",
    default => "anon",
  }
  return [$name, $props.items[0]][0]
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("props.items[0]"));
        assert!(js.contains("props.role === \"owner\""));
        assert!(js.contains("props.role === \"member\""));
        assert!(js.contains("props.role === \"user\""));
    }

    #[test]
    fn emits_named_arg_call_as_object_payload() {
        let source = r#"
function greet($props: object) {
  return $props.name
}
$value = greet(name: "Sami")
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("greet({\"name\": \"Sami\"})"));
    }

    #[test]
    fn emits_keyed_array_as_object_literal() {
        let source = r#"
$data = ["name" => "Sami", "role" => "owner"]
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("{\"name\": \"Sami\", \"role\": \"owner\"}"));
    }

    #[test]
    fn emits_method_and_nullsafe_access() {
        let source = r#"
$value = $user->getName()
$title = $user?->profile?->title
$nick = $user?->getNick()
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("user.getName()"));
        assert!(js.contains("(user)?.profile"));
        assert!(js.contains("(user)?.getNick()"));
    }

    #[test]
    fn emits_while_for_foreach_statements() {
        let source = r#"
$i = 0
while ($i < 2) {
  $i = $i + 1
}
for ($j = 0; $j < 2; $j = $j + 1) {
  $i = $i + $j
}
foreach ($items as $idx => $item) {
  $i = $i + $idx
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("while ((i < 2))"));
        assert!(js.contains("for (let j = 0; (j < 2); (j = (j + 1)))"));
        assert!(js.contains("for (const [idx , item] of Object.entries(items))"));
    }

    #[test]
    fn emits_break_continue_and_cast_helpers() {
        let source = r#"
for ($i = 0; $i < 10; $i = $i + 1) {
  if ($i == 2) {
    continue
  }
  if ($i == 8) {
    break
  }
}
$ok = isset($user.name)
$missing = empty($user.name)
$num = (int)"42"
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("continue;"));
        assert!(js.contains("if ((i === 2))"));
        assert!(js.contains("if ((i === 8))"));
        assert!(js.contains("break;"));
        assert!(js.contains("!== undefined"));
        assert!(js.contains("Number.parseInt(\"42\", 10)"));
    }

    #[test]
    fn emits_switch_and_dowhile_statements() {
        let source = r#"
$i = 0
do {
  $i = $i + 1
} while ($i < 2)
switch ($i) {
  case 1:
    $i = 10
    break
  default:
    $i = 20
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("do {"));
        assert!(js.contains("while ((i < 2));"));
        assert!(js.contains("switch (i)"));
        assert!(js.contains("case 1:"));
        assert!(js.contains("default:"));
    }

    #[test]
    fn emits_arrow_function_expression() {
        let source = r#"
$fn = fn ($x: int) => $x + 1
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("let fn = (x) => (x + 1);"));
    }

    #[test]
    fn emits_print_and_inline_html() {
        let source = r#"
print("hi")
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("console.log(\"hi\")"));
    }

    #[test]
    fn emits_closure_expression() {
        let source = r#"
$fn = function ($x: int) {
  return $x + 1
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("function(x)"));
        assert!(js.contains("return (x + 1);"));
    }

    #[test]
    fn emits_assign_ops_and_inc_dec() {
        let source = r#"
$counter = 1;
$counter += 2;
$counter--;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("let counter = 1;"));
        assert!(js.contains("counter += 2"));
        assert!(js.contains("counter--)"));
    }

    #[test]
    fn emits_throw_statement() {
        let source = r#"
throw "boom";
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("throw \"boom\";"));
    }

    #[test]
    fn emits_static_call_and_class_const_fetch() {
        let source = r#"
$value = Option::Some(1);
$status = Http::OK;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("Option.Some(1)"));
        assert!(js.contains("Http.OK"));
    }

    #[test]
    fn emits_struct_literal_as_object() {
        let source = r#"
$user = User { $id: 1, $name: "sam" };
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("\"__struct\": \"User\""));
        assert!(js.contains("\"id\": 1"));
        assert!(js.contains("\"name\": \"sam\""));
    }

    #[test]
    fn emits_new_expression() {
        let source = r#"
$point = new Point(1, 2);
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("new Point(1, 2)"));
    }

    #[test]
    fn emits_magic_constants() {
        let source = r#"
$line = __LINE__;
$file = __FILE__;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("let line = 0;"));
        assert!(js.contains("let file = \"\";"));
    }

    #[test]
    fn emits_assign_ref_as_assignment() {
        let source = r#"
$a = 1;
$b = &$a;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("let a = 1;"));
        assert!(js.contains("let b = a;"));
    }

    #[test]
    fn emits_include_with_runtime_stub() {
        let source = r#"
$value = include "./part.phpx";
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("function __phpx_include(path, kind)"));
        assert!(js.contains("__phpx_include(\"./part.phpx\", \"include\")"));
    }

    #[test]
    fn emits_const_statement() {
        let source = r#"
const ANSWER = 42;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("const ANSWER = 42;"));
    }

    #[test]
    fn emits_try_catch_finally_statement() {
        let source = r#"
try {
  throw "boom";
} catch (Exception $e) {
  $msg = $e;
} finally {
  $done = true;
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("try {"));
        assert!(js.contains("catch (e) {"));
        assert!(js.contains("finally {"));
    }

    #[test]
    fn emits_global_static_unset_statements() {
        let source = r#"
function demo($x: int): int {
  global $shared;
  static $count = 1;
  unset($shared);
  return $x;
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("global declarations are no-ops"));
        assert!(js.contains("let count = 1;"));
        assert!(js.contains("shared = undefined;"));
    }
    #[test]
    fn emits_label_goto_declare_and_halt_compiler_statements() {
        let source = r#"
start:
declare(ticks=1) {
  $x = 1;
}
goto start;
__halt_compiler();
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("label start ignored"));
        assert!(js.contains("goto start is not supported"));
        assert!(js.contains("let x = 1;"));
        assert!(js.contains("__halt_compiler ignored"));
    }

    #[test]
    fn emits_await_eval_and_clone_expressions() {
        let source = r#"
$one = await $promise;
$two = eval("40 + 2");
$three = clone $obj;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("await promise"));
        assert!(js.contains("eval(\"40 + 2\")"));
        assert!(js.contains("structuredClone(obj)"));
    }

    #[test]
    fn emits_die_and_exit_expressions() {
        let source = r#"
$one = die("boom");
$two = exit();
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let js = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect("subset emit");
        assert!(js.contains("throw new Error(String(\"boom\"))"));
        assert!(js.contains("throw new Error(\"exit\")"));
    }

    #[test]
    fn reports_specific_error_for_shell_exec_expression() {
        let source = r#"
$out = `echo hi`;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let err = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect_err("subset emit should fail");
        assert!(err.contains("shell execution is not supported"));
    }

    #[test]
    fn reports_specific_error_for_yield_expression() {
        let source = r#"
function gen() {
  yield 1;
}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let err = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect_err("subset emit should fail");
        assert!(err.contains("yield expressions are not supported"));
    }

    #[test]
    fn reports_specific_error_for_namespace_statement() {
        let source = r#"
namespace Demo;
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let err = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect_err("subset emit should fail");
        assert!(err.contains("namespace declarations are not supported"));
    }

    #[test]
    fn reports_specific_error_for_class_statement() {
        let source = r#"
class User {}
"#;
        let arena = Bump::new();
        let mut parser =
            Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
        let program = parser.parse_program();
        let err = emit_js_from_ast(&program, source.as_bytes(), SourceModuleMeta::empty())
            .expect_err("subset emit should fail");
        assert!(err.contains("class-like declarations are not supported"));
    }


    #[test]
    fn project_root_requires_deka_json() {
        let tmp = tempfile::tempdir().expect("tmp");
        let input = tmp.path().join("src").join("main.phpx");
        std::fs::create_dir_all(input.parent().expect("parent")).expect("mkdir");
        std::fs::write(&input, "<div />").expect("write");

        let err = resolve_project_root(&input).expect_err("should fail without deka.json");
        assert!(err.contains("deka.json"));
    }

    #[test]
    fn ensure_project_layout_requires_lock_file() {
        let tmp = tempfile::tempdir().expect("tmp");
        std::fs::write(tmp.path().join("deka.json"), "{}").expect("deka.json");
        let meta = SourceModuleMeta::empty();
        let err = ensure_project_layout(tmp.path(), &meta).expect_err("missing lock");
        assert!(err.contains("deka.lock"));
    }

    #[test]
    fn ensure_project_layout_requires_php_modules_for_stdlib_imports() {
        let tmp = tempfile::tempdir().expect("tmp");
        std::fs::write(tmp.path().join("deka.json"), "{}").expect("deka.json");
        std::fs::write(tmp.path().join("deka.lock"), "{}").expect("deka.lock");
        let source = "---\nimport { parse } from 'encoding/json'\n---\n<div />\n";
        let meta = parse_source_module_meta(source);
        let err = ensure_project_layout(tmp.path(), &meta).expect_err("missing php_modules");
        assert!(err.contains("php_modules"));
    }

    #[test]
    fn ensure_project_layout_requires_stdlib_module_files() {
        let tmp = tempfile::tempdir().expect("tmp");
        std::fs::write(tmp.path().join("deka.json"), "{}").expect("deka.json");
        std::fs::write(tmp.path().join("deka.lock"), "{}").expect("deka.lock");
        std::fs::create_dir_all(tmp.path().join("php_modules")).expect("php_modules");
        let source = "---\nimport { parse } from 'encoding/json'\n---\n<div />\n";
        let meta = parse_source_module_meta(source);
        let err = ensure_project_layout(tmp.path(), &meta).expect_err("missing stdlib module file");
        assert!(err.contains("encoding/json"));
    }

    #[test]
    fn ensure_project_layout_accepts_present_stdlib_module_file() {
        let tmp = tempfile::tempdir().expect("tmp");
        std::fs::write(tmp.path().join("deka.json"), "{}").expect("deka.json");
        std::fs::write(tmp.path().join("deka.lock"), "{}").expect("deka.lock");
        std::fs::create_dir_all(tmp.path().join("php_modules").join("encoding")).expect("encoding dir");
        std::fs::write(
            tmp.path().join("php_modules").join("encoding").join("json.phpx"),
            "export function parse($v: string): object { return {} }",
        )
        .expect("json.phpx");
        let source = "---\nimport { parse } from 'encoding/json'\n---\n<div />\n";
        let meta = parse_source_module_meta(source);
        ensure_project_layout(tmp.path(), &meta).expect("layout should pass");
    }

    #[test]
    fn ensure_web_project_layout_requires_type_serve() {
        let tmp = tempfile::tempdir().expect("tmp");
        std::fs::write(
            tmp.path().join("deka.json"),
            "{\"type\":\"lib\",\"serve\":{\"entry\":\"app/main.phpx\"}}",
        )
        .expect("deka.json");
        std::fs::write(tmp.path().join("deka.lock"), "{}").expect("deka.lock");
        std::fs::create_dir_all(tmp.path().join("app")).expect("app");
        std::fs::create_dir_all(tmp.path().join("public")).expect("public");
        std::fs::write(tmp.path().join("public").join("index.html"), "<html></html>").expect("index");

        let err = ensure_web_project_layout(tmp.path()).expect_err("type should be required");
        assert!(err.contains("type=\"serve\""));
    }

    #[test]
    fn ensure_web_project_layout_accepts_type_serve() {
        let tmp = tempfile::tempdir().expect("tmp");
        std::fs::write(
            tmp.path().join("deka.json"),
            "{\"type\":\"serve\",\"serve\":{\"entry\":\"app/main.phpx\"}}",
        )
        .expect("deka.json");
        std::fs::write(tmp.path().join("deka.lock"), "{}").expect("deka.lock");
        std::fs::create_dir_all(tmp.path().join("app")).expect("app");
        std::fs::create_dir_all(tmp.path().join("public")).expect("public");
        std::fs::write(tmp.path().join("public").join("index.html"), "<html></html>").expect("index");

        ensure_web_project_layout(tmp.path()).expect("web layout should pass");
    }

    #[test]
    fn import_map_path_uses_output_directory() {
        let path = resolve_import_map_path(Path::new("dist/home.js"));
        assert_eq!(path, PathBuf::from("dist/importmap.json"));
    }

    #[test]
    fn import_map_adds_fallback_for_unmapped_bare_specifiers() {
        let source = "---
import { thing } from 'acme/widgets'
---
<div />
";
        let meta = parse_source_module_meta(source);
        let json = emit_import_map_json(&meta, Path::new("dist/home.js"));
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        let imports = value
            .get("imports")
            .and_then(|v| v.as_object())
            .expect("imports object");

        assert_eq!(
            imports.get("acme/widgets").and_then(|v| v.as_str()),
            Some("php_modules/acme/widgets.js")
        );
        assert_eq!(
            imports.get("component/").and_then(|v| v.as_str()),
            Some("/php_modules/component/")
        );
    }

    #[test]
    fn import_map_skips_relative_and_url_imports() {
        let source = "---
import { x } from './local.js'
import { y } from 'https://cdn.example/x.js'
---
<div />
";
        let meta = parse_source_module_meta(source);
        let json = emit_import_map_json(&meta, Path::new("dist/home.js"));
        let value: serde_json::Value = serde_json::from_str(&json).expect("json");
        let imports = value
            .get("imports")
            .and_then(|v| v.as_object())
            .expect("imports object");

        assert!(!imports.contains_key("./local.js"));
        assert!(!imports.contains_key("https://cdn.example/x.js"));
    }


}
