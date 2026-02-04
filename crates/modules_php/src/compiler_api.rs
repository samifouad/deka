//! PHPX compiler/validation entry points for tooling (LSP, tests, etc.).

use bumpalo::Bump;
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};

use crate::validation::exports::validate_exports;
use crate::validation::generics::validate_generics;
use crate::validation::imports::validate_imports;
use crate::validation::imports::{frontmatter_bounds, strip_php_tags_inline};
use crate::validation::jsx::{
    validate_components, validate_frontmatter, validate_jsx_expressions, validate_jsx_syntax,
    validate_template_section,
};
use crate::validation::modules::{validate_module_resolution, validate_wasm_imports};
use crate::validation::patterns::validate_match_exhaustiveness;
use crate::validation::syntax::validate_syntax;
use crate::validation::type_checker::check_types;
use crate::validation::type_syntax::validate_type_annotations;
use crate::validation::phpx_rules::{
    validate_no_exceptions, validate_no_namespace, validate_no_null, validate_no_oop,
};
use crate::validation::structs::{validate_struct_definitions, validate_struct_literals};
use crate::validation::ValidationResult;

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

    let type_errors = check_types(&program, source, Some(file_path));
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

    ValidationResult {
        errors,
        warnings,
        ast: if has_parse_errors { None } else { Some(program) },
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
