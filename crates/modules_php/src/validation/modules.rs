use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{ErrorKind, Severity, ValidationError};
use crate::validation::exports::{parse_export_function, parse_export_list_line};
use crate::validation::imports::{
    consume_comment_line, frontmatter_bounds, parse_import_line, strip_php_tags_inline, ImportKind,
    ImportSpec,
};

#[derive(Debug, Clone)]
struct ModuleNode {
    module_id: String,
    imports: Vec<ImportEdge>,
    exports: HashSet<String>,
}

#[derive(Debug, Clone)]
struct ImportEdge {
    module_id: String,
    imported: String,
    line: usize,
    column: usize,
    raw_from: String,
}

pub fn validate_module_resolution(source: &str, file_path: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let modules_root = resolve_modules_root(file_path);
    let imports = collect_import_specs(source, file_path);
    let available_modules = modules_root
        .as_deref()
        .map(scan_phpx_modules)
        .unwrap_or_default();

    let mut graph = ModuleGraph::new(modules_root.clone(), available_modules.clone());
    if !imports.is_empty() {
        graph.ensure_loaded("<entry>", Path::new(file_path), &mut errors);
    }

    if !errors.is_empty() {
        return errors;
    }

    graph.collect_missing_exports(&mut errors);
    graph.detect_cycles(&mut errors);
    errors
}

pub fn validate_wasm_imports(source: &str, file_path: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let modules_root = resolve_modules_root(file_path);
    let imports = collect_import_specs(source, file_path);
    let available_wasm = modules_root
        .as_deref()
        .map(scan_wasm_modules)
        .unwrap_or_default();

    for spec in imports {
        if spec.kind != ImportKind::Wasm {
            continue;
        }
        if spec.from.starts_with('@') && !spec.from.starts_with("@/") && !is_valid_user_module(&spec.from) {
            errors.push(wasm_error(
                spec.line,
                spec.column,
                spec.from.len().max(1),
                format!("Invalid wasm module id '{}'.", spec.from),
                "Use '@user/module' format for user wasm modules.",
            ));
            continue;
        }
        match resolve_wasm_target(
            &spec.from,
            file_path,
            modules_root.as_deref(),
            Some(&available_wasm),
        ) {
            Ok(target) => {
                validate_wasm_manifest(&target, &spec, &mut errors);
            }
            Err(err) => errors.push(err),
        }
    }

    errors
}

struct ModuleGraph {
    modules_root: Option<PathBuf>,
    available_modules: HashSet<String>,
    nodes: HashMap<String, ModuleNode>,
}

impl ModuleGraph {
    fn new(modules_root: Option<PathBuf>, available_modules: HashSet<String>) -> Self {
        Self {
            modules_root,
            available_modules,
            nodes: HashMap::new(),
        }
    }

    fn ensure_loaded(
        &mut self,
        module_id: &str,
        file_path: &Path,
        errors: &mut Vec<ValidationError>,
    ) {
        if self.nodes.contains_key(module_id) {
            return;
        }
        let source = match std::fs::read_to_string(file_path) {
            Ok(src) => src,
            Err(err) => {
                errors.push(module_error(
                    1,
                    1,
                    1,
                    format!(
                        "Failed to read module '{}': {}",
                        module_id,
                        err
                    ),
                    "Ensure the module file exists and is readable.",
                ));
                return;
            }
        };

        let mut imports = Vec::new();
        let import_specs = collect_import_specs(&source, file_path.to_string_lossy().as_ref());
        for spec in import_specs {
            if spec.kind == ImportKind::Wasm {
                continue;
            }
            if spec.from.starts_with('@') && !spec.from.starts_with("@/") && !is_valid_user_module(&spec.from) {
                errors.push(module_error(
                    spec.line,
                    spec.column,
                    spec.from.len().max(1),
                    format!("Invalid module id '{}'.", spec.from),
                    "Use '@user/module' format for user modules.",
                ));
                continue;
            }
            match resolve_import_target(
                &spec.from,
                file_path.to_string_lossy().as_ref(),
                self.modules_root.as_deref(),
                Some(&self.available_modules),
            ) {
                Ok(resolved) => {
                    imports.push(ImportEdge {
                        module_id: resolved.module_id.clone(),
                        imported: spec.imported.clone(),
                        line: spec.line,
                        column: spec.column,
                        raw_from: spec.from.clone(),
                    });
                    self.ensure_loaded(&resolved.module_id, &resolved.file_path, errors);
                }
                Err(err) => errors.push(err),
            }
        }

        let mut exports = collect_exports(&source, file_path.to_string_lossy().as_ref());
        if is_template_module(&source) {
            exports.insert("Component".to_string());
        }

        self.nodes.insert(
            module_id.to_string(),
            ModuleNode {
                module_id: module_id.to_string(),
                imports,
                exports,
            },
        );
    }

    fn collect_missing_exports(&self, errors: &mut Vec<ValidationError>) {
        for node in self.nodes.values() {
            for edge in &node.imports {
                let Some(target) = self.nodes.get(&edge.module_id) else {
                    errors.push(module_error(
                        edge.line,
                        edge.column,
                        edge.raw_from.len().max(1),
                        format!(
                            "Unknown phpx module '{}' imported by '{}'.",
                            edge.raw_from, node.module_id
                        ),
                        "Ensure the module exists in php_modules/.",
                    ));
                    continue;
                };
                if !target.exports.contains(&edge.imported) {
                    errors.push(module_error(
                        edge.line,
                        edge.column,
                        edge.imported.len().max(1),
                        format!(
                            "Missing export '{}' in '{}' (imported by '{}').",
                            edge.imported, edge.raw_from, node.module_id
                        ),
                        "Export the symbol from the module or update the import.",
                    ));
                }
            }
        }
    }

    fn detect_cycles(&self, errors: &mut Vec<ValidationError>) {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();

        for module_id in self.nodes.keys() {
            self.visit(module_id, &mut visiting, &mut visited, errors);
        }
    }

    fn visit(
        &self,
        module_id: &str,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        if visited.contains(module_id) {
            return;
        }
        if visiting.contains(module_id) {
            errors.push(module_error(
                1,
                1,
                module_id.len().max(1),
                format!("Cyclic phpx import detected at '{}'.", module_id),
                "Break the cycle by removing one of the imports.",
            ));
            return;
        }
        visiting.insert(module_id.to_string());
        if let Some(node) = self.nodes.get(module_id) {
            for edge in &node.imports {
                self.visit(&edge.module_id, visiting, visited, errors);
            }
        }
        visiting.remove(module_id);
        visited.insert(module_id.to_string());
    }
}

fn collect_import_specs(source: &str, file_path: &str) -> Vec<ImportSpec> {
    let lines: Vec<&str> = source.lines().collect();
    let bounds = frontmatter_bounds(&lines);
    let scan_end = bounds.map(|(_, end)| end).unwrap_or(lines.len());
    let mut in_block_comment = false;
    let mut specs = Vec::new();
    for (idx, line) in lines.iter().enumerate().take(scan_end) {
        if let Some((start, end)) = bounds {
            if idx == start || idx == end {
                continue;
            }
        }
        let clean = strip_php_tags_inline(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() {
            continue;
        }
        if consume_comment_line(trimmed, &mut in_block_comment) {
            continue;
        }
        if trimmed.starts_with("import ") {
            if let Ok(mut parsed) = parse_import_line(trimmed, line, idx + 1, file_path) {
                specs.append(&mut parsed);
            }
        }
    }
    specs
}

fn collect_exports(source: &str, file_path: &str) -> HashSet<String> {
    let lines: Vec<&str> = source.lines().collect();
    let bounds = frontmatter_bounds(&lines);
    let scan_end = bounds.map(|(_, end)| end).unwrap_or(lines.len());
    let mut in_block_comment = false;
    let mut exports = HashSet::new();
    for (idx, line) in lines.iter().enumerate().take(scan_end) {
        if let Some((start, end)) = bounds {
            if idx == start || idx == end {
                continue;
            }
        }
        let clean = strip_php_tags_inline(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() {
            continue;
        }
        if consume_comment_line(trimmed, &mut in_block_comment) {
            continue;
        }
        if trimmed.starts_with("export function") {
            if let Ok(spec) = parse_export_function(trimmed, line, idx + 1, file_path) {
                exports.insert(spec.name);
            }
            continue;
        }
        if trimmed.starts_with("export {") {
            if let Ok(specs) = parse_export_list_line(trimmed, line, idx + 1, file_path) {
                for spec in specs {
                    exports.insert(spec.name);
                }
            }
        }
    }
    exports
}

fn is_template_module(source: &str) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    let bounds = frontmatter_bounds(&lines);
    let Some((_, end)) = bounds else {
        return false;
    };
    lines
        .iter()
        .skip(end + 1)
        .any(|line| !line.trim().is_empty())
}

pub(crate) fn resolve_modules_root(file_path: &str) -> Option<PathBuf> {
    let path = Path::new(file_path);
    let dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    if let Some(root) = find_project_root(&dir) {
        let candidate = root.join("php_modules");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    for ancestor in dir.ancestors() {
        if ancestor.file_name().is_some_and(|name| name == "php_modules") {
            return Some(ancestor.to_path_buf());
        }
        let candidate = ancestor.join("php_modules");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn scan_phpx_modules(modules_root: &Path) -> HashSet<String> {
    let mut modules = HashSet::new();
    let mut stack = vec![modules_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == ".cache" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("phpx") {
                if let Ok(rel) = path.strip_prefix(modules_root) {
                    let rel = rel.to_string_lossy().replace('\\', "/");
                    modules.insert(module_id_from_rel(&rel));
                }
            }
        }
    }
    modules
}

fn scan_wasm_modules(modules_root: &Path) -> HashSet<String> {
    let mut modules = HashSet::new();
    let mut stack = vec![modules_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == ".cache" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if name == "deka.json" {
                if let Some(parent) = path.parent() {
                    if let Ok(rel) = parent.strip_prefix(modules_root) {
                        let rel = rel.to_string_lossy().replace('\\', "/");
                        if !rel.is_empty() {
                            modules.insert(rel);
                        }
                    }
                }
            }
        }
    }
    modules
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        if ancestor.join("deka.lock").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

struct ResolvedImportTarget {
    module_id: String,
    file_path: PathBuf,
}

fn resolve_import_target(
    raw: &str,
    current_file_path: &str,
    modules_root: Option<&Path>,
    available_modules: Option<&HashSet<String>>,
) -> Result<ResolvedImportTarget, ValidationError> {
    let raw = raw.trim();
    let is_relative = raw.starts_with('.');
    let is_project_alias = raw.starts_with("@/");
    let spec_path = raw.strip_prefix("@/").unwrap_or(raw);
    let base_dir = if is_relative {
        Path::new(current_file_path)
            .parent()
            .map(PathBuf::from)
    } else if is_project_alias {
        modules_root
            .and_then(|root| root.parent().map(PathBuf::from))
    } else {
        modules_root.map(PathBuf::from)
    }
    .ok_or_else(|| {
        module_error(
            1,
            1,
            raw.len().max(1),
            format!(
                "Missing php_modules for import '{}' in {}.",
                raw, current_file_path
            ),
            "Create php_modules/ or run `deka init`.",
        )
    })?;

    let base_path = base_dir.join(spec_path);
    let mut candidates = Vec::new();
    if raw.ends_with(".phpx") {
        candidates.push(base_path.clone());
    } else {
        candidates.push(base_path.with_extension("phpx"));
        candidates.push(base_path.join("index.phpx"));
    }
    if !is_relative && !is_project_alias {
        if let Some(root) = modules_root {
            candidates.push(root.join(format!("{raw}.phpx")));
            candidates.push(root.join(raw).join("index.phpx"));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            if is_project_alias {
                if let Some(project_root) = modules_root.and_then(|root| root.parent()) {
                    if let Ok(rel) = candidate.strip_prefix(project_root) {
                        let rel = rel.to_string_lossy().replace('\\', "/");
                        let module_id = format!("@/{}", module_id_from_rel(&rel));
                        return Ok(ResolvedImportTarget {
                            module_id,
                            file_path: candidate,
                        });
                    }
                }
            } else if let Some(root) = modules_root {
                if let Ok(rel) = candidate.strip_prefix(root) {
                    let rel = rel.to_string_lossy().replace('\\', "/");
                    let module_id = module_id_from_rel(&rel);
                    return Ok(ResolvedImportTarget {
                        module_id,
                        file_path: candidate,
                    });
                }
            }
            return Ok(ResolvedImportTarget {
                module_id: raw.to_string(),
                file_path: candidate,
            });
        }
    }

    Err(module_error(
        1,
        1,
        raw.len().max(1),
        format!(
            "Missing phpx module '{}' (imported from {}).",
            raw, current_file_path
        ),
        available_modules
            .and_then(|modules| format_available_modules(modules, "Available modules: "))
            .as_deref()
            .unwrap_or("Ensure the module exists in php_modules/."),
    ))
}

struct ResolvedWasmTarget {
    root_path: PathBuf,
    manifest_path: PathBuf,
}

fn resolve_wasm_target(
    raw: &str,
    current_file_path: &str,
    modules_root: Option<&Path>,
    available_wasm: Option<&HashSet<String>>,
) -> Result<ResolvedWasmTarget, ValidationError> {
    let raw = raw.trim();
    let modules_root = modules_root.ok_or_else(|| {
        wasm_error(
            1,
            1,
            raw.len().max(1),
            format!(
                "Wasm import requires php_modules/ (missing for {}).",
                current_file_path
            ),
            "Create php_modules/ or run `deka init`.",
        )
    })?;

    let is_relative = raw.starts_with('.');
    let is_project_alias = raw.starts_with("@/");
    let spec_path = raw.strip_prefix("@/").unwrap_or(raw);
    let base_dir = if is_relative {
        Path::new(current_file_path)
            .parent()
            .unwrap_or(modules_root)
            .to_path_buf()
    } else if is_project_alias {
        modules_root
            .parent()
            .unwrap_or(modules_root)
            .to_path_buf()
    } else {
        modules_root.to_path_buf()
    };
    let root_path = base_dir.join(spec_path);
    let allowed_root = if is_project_alias {
        modules_root.parent().unwrap_or(modules_root)
    } else {
        modules_root
    };
    let rel = root_path
        .strip_prefix(allowed_root)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"));
    if rel.as_deref().unwrap_or("").starts_with("..") || rel.is_none() {
        return Err(wasm_error(
            1,
            1,
            raw.len().max(1),
            format!(
                "Wasm import must resolve inside {} ({}: {}).",
                if is_project_alias { "project root" } else { "php_modules/" },
                current_file_path,
                raw
            ),
            if is_project_alias {
                "Move the wasm module under the project root."
            } else {
                "Move the wasm module under php_modules/."
            },
        ));
    }

    let manifest_path = root_path.join("deka.json");
    if !manifest_path.exists() {
        let help = available_wasm
            .and_then(|modules| format_available_modules(modules, "Available WASM modules: "));
        return Err(wasm_error(
            1,
            1,
            raw.len().max(1),
            format!(
                "Missing wasm module manifest for '{}' (expected {}).",
                raw,
                manifest_path.display()
            ),
            help.as_deref()
                .unwrap_or("Add deka.json to the wasm module directory."),
        ));
    }

    Ok(ResolvedWasmTarget {
        root_path,
        manifest_path,
    })
}

fn validate_wasm_manifest(
    target: &ResolvedWasmTarget,
    spec: &ImportSpec,
    errors: &mut Vec<ValidationError>,
) {
    let raw = match std::fs::read_to_string(&target.manifest_path) {
        Ok(raw) => raw,
        Err(err) => {
            errors.push(wasm_error(
                spec.line,
                spec.column,
                spec.from.len().max(1),
                format!(
                    "Failed to read wasm manifest {}: {}",
                    target.manifest_path.display(),
                    err
                ),
                "Ensure the manifest is readable JSON.",
            ));
            return;
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(err) => {
            errors.push(wasm_error(
                spec.line,
                spec.column,
                spec.from.len().max(1),
                format!(
                    "Invalid wasm manifest {}: {}",
                    target.manifest_path.display(),
                    err
                ),
                "Fix the JSON in deka.json.",
            ));
            return;
        }
    };
    let module_path = parsed
        .get("module")
        .and_then(|v| v.as_str())
        .unwrap_or("module.wasm");
    let module_path = target.root_path.join(module_path);
    if !module_path.exists() {
        errors.push(wasm_error(
            spec.line,
            spec.column,
            spec.from.len().max(1),
            format!(
                "Missing wasm module binary {}.",
                module_path.display()
            ),
            "Build the wasm module or update deka.json.",
        ));
    }

    let stub_path = parsed
        .get("stubs")
        .and_then(|v| v.as_str())
        .map(|s| target.root_path.join(s))
        .unwrap_or_else(|| target.root_path.join("module.d.phpx"));
    if !stub_path.exists() {
        errors.push(wasm_error(
            spec.line,
            spec.column,
            spec.from.len().max(1),
            format!(
                "Missing wasm stub file {}.",
                stub_path.display()
            ),
            "Generate stubs with `deka wasm stubs`.",
        ));
        return;
    }

    let stub_source = match std::fs::read_to_string(&stub_path) {
        Ok(src) => src,
        Err(err) => {
            errors.push(wasm_error(
                spec.line,
                spec.column,
                spec.from.len().max(1),
                format!(
                    "Failed to read wasm stub {}: {}",
                    stub_path.display(),
                    err
                ),
                "Ensure the stub file is readable.",
            ));
            return;
        }
    };
    let exports = collect_exports(&stub_source, stub_path.to_string_lossy().as_ref());
    if !exports.contains(&spec.imported) {
        errors.push(wasm_error(
            spec.line,
            spec.column,
            spec.imported.len().max(1),
            format!(
                "Missing wasm export '{}' in {}.",
                spec.imported,
                stub_path.display()
            ),
            "Regenerate stubs or update the import to match exported names.",
        ));
    }
}

fn module_id_from_rel(rel: &str) -> String {
    let normalized = rel.replace('\\', "/");
    if normalized.ends_with("/index.phpx") {
        return normalized.replace("/index.phpx", "");
    }
    normalized.trim_end_matches(".phpx").to_string()
}

fn is_valid_user_module(raw: &str) -> bool {
    if !raw.starts_with('@') {
        return true;
    }
    let parts: Vec<&str> = raw.split('/').collect();
    parts.len() == 2
        && parts[0].len() > 1
        && !parts[1].is_empty()
        && parts.iter().all(|part| !part.trim().is_empty())
}

fn format_available_modules(
    modules: &HashSet<String>,
    prefix: &str,
) -> Option<String> {
    if modules.is_empty() {
        return None;
    }
    let mut list: Vec<String> = modules.iter().cloned().collect();
    list.sort();
    let preview: Vec<String> = list.into_iter().take(12).collect();
    Some(format!("{}{}", prefix, preview.join(", ")))
}

fn module_error(
    line: usize,
    column: usize,
    underline_length: usize,
    message: String,
    help_text: &str,
) -> ValidationError {
    ValidationError {
        kind: ErrorKind::ModuleError,
        line,
        column,
        message,
        help_text: help_text.to_string(),
        suggestion: None,
        underline_length: underline_length.max(1),
        severity: Severity::Error,
    }
}

fn wasm_error(
    line: usize,
    column: usize,
    underline_length: usize,
    message: String,
    help_text: &str,
) -> ValidationError {
    ValidationError {
        kind: ErrorKind::WasmError,
        line,
        column,
        message,
        help_text: help_text.to_string(),
        suggestion: None,
        underline_length: underline_length.max(1),
        severity: Severity::Error,
    }
}
