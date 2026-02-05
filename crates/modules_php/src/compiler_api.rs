//! PHPX compiler/validation entry points for tooling (LSP, tests, etc.).

use bumpalo::Bump;
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};

use crate::validation::exports::validate_exports;
use crate::validation::generics::validate_generics;
use crate::validation::imports::validate_imports;
use crate::validation::imports::{
    consume_comment_line, frontmatter_bounds, parse_import_line, strip_php_tags_inline, ImportKind,
    ImportSpec,
};
use crate::validation::jsx::{
    validate_components, validate_frontmatter, validate_jsx_expressions, validate_jsx_syntax,
    validate_template_section,
};
use crate::validation::modules::{resolve_modules_root, validate_module_resolution, validate_wasm_imports};
use crate::validation::patterns::validate_match_exhaustiveness;
use crate::validation::syntax::validate_syntax;
use crate::validation::type_checker::{check_types, check_types_with_externals};
use crate::validation::type_syntax::validate_type_annotations;
use crate::validation::phpx_rules::{
    validate_no_exceptions, validate_no_namespace, validate_no_null, validate_no_oop,
};
use crate::validation::structs::{validate_struct_definitions, validate_struct_literals};
use crate::validation::{ErrorKind, Severity, ValidationError, ValidationResult};
use php_rs::phpx::typeck::{external_functions_from_stub, ExternalFunctionSig};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Compile and validate a PHPX source file.
///
/// Returns a `ValidationResult` with errors, warnings, and the parsed AST
/// (if no syntax errors were encountered). Callers should provide a bump
/// arena for AST allocations.
pub fn compile_phpx<'a>(source: &str, file_path: &str, arena: &'a Bump) -> ValidationResult<'a> {
    let parser_source = preprocess_phpx_source(source);
    let lexer = Lexer::new(parser_source.as_bytes());
    let mut parser = Parser::new_with_mode(lexer, arena, ParserMode::Phpx);
    let program = parser.parse_program();

    let mut errors = validate_syntax(source, &program);
    let mut warnings = Vec::new();
    let has_parse_errors = !errors.is_empty();

    let (import_errors, import_warnings) = validate_imports(source, file_path);
    errors.extend(import_errors);
    warnings.extend(import_warnings);

    let export_errors = validate_exports(source, file_path, &program);
    errors.extend(export_errors);

    let type_errors = validate_type_annotations(&program, source);
    errors.extend(type_errors);

    let (mut wasm_functions, wasm_errors) =
        collect_wasm_stub_signatures(source, file_path, arena);
    errors.extend(wasm_errors);

    let type_errors = if wasm_functions.is_empty() {
        check_types(&program, source, Some(file_path))
    } else {
        check_types_with_externals(&program, source, Some(file_path), &wasm_functions)
    };
    errors.extend(type_errors);

    let (generic_errors, generic_warnings) = validate_generics(&program, source);
    errors.extend(generic_errors);
    warnings.extend(generic_warnings);

    errors.extend(validate_no_null(&program, source));
    errors.extend(validate_no_exceptions(&program, source));
    errors.extend(validate_no_oop(&program, source));
    errors.extend(validate_no_namespace(&program, source));

    errors.extend(validate_struct_definitions(&program, source));
    errors.extend(validate_struct_literals(&program, source));

    errors.extend(validate_frontmatter(source, file_path));
    errors.extend(validate_template_section(source, file_path));
    errors.extend(validate_jsx_syntax(&program, source));
    errors.extend(validate_jsx_expressions(&program, source));
    errors.extend(validate_components(&program, source));

    errors.extend(validate_module_resolution(source, file_path));
    errors.extend(validate_wasm_imports(source, file_path));

    errors.extend(validate_match_exhaustiveness(&program, source));

    if has_parse_errors {
        wasm_functions.clear();
    }

    ValidationResult {
        errors,
        warnings,
        ast: if has_parse_errors { None } else { Some(program) },
        wasm_functions,
    }
}

fn preprocess_phpx_source(source: &str) -> String {
    let line_refs: Vec<&str> = source.lines().collect();
    let bounds = frontmatter_bounds(&line_refs);
    let mut output = String::with_capacity(source.len());
    let mut line_index = 0usize;

    for segment in source.split_inclusive('\n') {
        let in_frontmatter = if let Some((start, end)) = bounds {
            line_index > start && line_index < end
        } else {
            true
        };
        let is_delim = bounds
            .map(|(start, end)| line_index == start || line_index == end)
            .unwrap_or(false);

        let clean = strip_php_tags_inline(segment);
        let trimmed = clean.trim();

        let mut masked = false;
        if !in_frontmatter || is_delim {
            masked = true;
        } else if trimmed.starts_with("import ") {
            masked = true;
        } else if trimmed.starts_with("export {") {
            masked = true;
        } else if trimmed.starts_with("export ") && !trimmed.starts_with("export function") {
            masked = true;
        }

        if masked {
            output.push_str(&mask_segment(segment));
        } else if trimmed.starts_with("export function") {
            output.push_str(&mask_export_keyword(segment));
        } else {
            output.push_str(segment);
        }

        line_index += 1;
    }

    output
}

fn mask_segment(segment: &str) -> String {
    segment
        .chars()
        .map(|ch| if ch == '\n' { '\n' } else { ' ' })
        .collect()
}

fn mask_export_keyword(segment: &str) -> String {
    if let Some(idx) = segment.find("export") {
        let mut out = String::with_capacity(segment.len());
        out.push_str(&segment[..idx]);
        out.push_str("      ");
        out.push_str(&segment[idx + 6..]);
        return out;
    }
    segment.to_string()
}

fn collect_wasm_stub_signatures(
    source: &str,
    file_path: &str,
    arena: &Bump,
) -> (HashMap<String, ExternalFunctionSig>, Vec<ValidationError>) {
    let mut errors = Vec::new();
    let specs = collect_wasm_import_specs(source, file_path);
    if specs.is_empty() {
        return (HashMap::new(), errors);
    }

    let mut out: HashMap<String, ExternalFunctionSig> = HashMap::new();
    let mut stub_cache: HashMap<PathBuf, HashMap<String, ExternalFunctionSig>> = HashMap::new();

    for spec in specs {
        let Some(stub_path) = resolve_wasm_stub_path(file_path, &spec) else {
            continue;
        };
        let entry = if let Some(cached) = stub_cache.get(&stub_path) {
            Some(cached)
        } else {
            let stub_source = match std::fs::read_to_string(&stub_path) {
                Ok(src) => src,
                Err(err) => {
                    errors.push(wasm_stub_error(
                        spec.line,
                        spec.column,
                        spec.from.len().max(1),
                        format!(
                            "Failed to read wasm stub {}: {}",
                            stub_path.display(),
                            err
                        ),
                    ));
                    continue;
                }
            };
            let program = parse_stub_program(&stub_source, arena);
            if !program.errors.is_empty() {
                let mut parse_errors =
                    crate::validation::parse_errors_to_validation_errors(&stub_source, program.errors);
                for err in &mut parse_errors {
                    err.kind = ErrorKind::WasmError;
                    err.message = format!("Wasm stub parse error: {}", err.message);
                    err.help_text = "Regenerate stubs with `deka wasm stubs`.".to_string();
                }
                errors.extend(parse_errors);
                continue;
            }
            let sigs = external_functions_from_stub(&program, stub_source.as_bytes());
            stub_cache.insert(stub_path.clone(), sigs);
            stub_cache.get(&stub_path)
        };

        let Some(exports) = entry else {
            continue;
        };
        if let Some(sig) = exports.get(&spec.imported) {
            out.insert(spec.local.clone(), sig.clone());
        }
    }

    (out, errors)
}

fn wasm_stub_error(
    line: usize,
    column: usize,
    underline_length: usize,
    message: String,
) -> ValidationError {
    ValidationError {
        kind: ErrorKind::WasmError,
        line,
        column,
        message,
        help_text: "Regenerate stubs with `deka wasm stubs`.".to_string(),
        suggestion: None,
        underline_length,
        severity: Severity::Error,
    }
}

fn collect_wasm_import_specs(source: &str, file_path: &str) -> Vec<ImportSpec> {
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
        .into_iter()
        .filter(|spec| spec.kind == ImportKind::Wasm)
        .collect()
}

fn resolve_wasm_stub_path(file_path: &str, spec: &ImportSpec) -> Option<PathBuf> {
    let modules_root = resolve_modules_root(file_path)?;
    let raw = spec.from.trim();
    let is_relative = raw.starts_with('.');
    let base_dir = if is_relative {
        Path::new(file_path)
            .parent()
            .unwrap_or(modules_root.as_path())
            .to_path_buf()
    } else {
        modules_root.clone()
    };
    let root_path = base_dir.join(raw);
    if root_path.strip_prefix(&modules_root).ok().is_none() {
        return None;
    }
    let manifest_path = root_path.join("deka.json");
    let raw = std::fs::read_to_string(&manifest_path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let stub_rel = parsed
        .get("stubs")
        .and_then(|v| v.as_str())
        .unwrap_or("module.d.phpx");
    Some(root_path.join(stub_rel))
}

fn parse_stub_program<'a>(source: &str, arena: &'a Bump) -> php_rs::parser::ast::Program<'a> {
    let processed = preprocess_stub_source(source);
    let lexer = Lexer::new(processed.as_bytes());
    let mut parser = Parser::new_with_mode(lexer, arena, ParserMode::Phpx);
    parser.parse_program()
}

fn preprocess_stub_source(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    for segment in source.split_inclusive('\n') {
        let clean = strip_php_tags_inline(segment);
        let trimmed = clean.trim_start();
        if trimmed.starts_with("export ") {
            if let Some(idx) = clean.find("export") {
                let mut out = String::with_capacity(clean.len());
                out.push_str(&clean[..idx]);
                out.push_str("      ");
                out.push_str(&clean[idx + 6..]);
                output.push_str(&out);
            } else {
                output.push_str(&clean);
            }
        } else {
            output.push_str(&clean);
        }
    }
    output
}
