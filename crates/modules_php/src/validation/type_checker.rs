use std::path::Path;

use php_rs::parser::ast::Program;
use php_rs::phpx::typeck::{
    check_program_with_path, check_program_with_path_and_externals, ExternalFunctionSig,
    TypeError as PhpTypeError,
};

use super::{ErrorKind, Severity, ValidationError};

pub fn check_types(program: &Program, source: &str, file_path: Option<&str>) -> Vec<ValidationError> {
    let path = file_path
        .filter(|path| !path.is_empty())
        .map(Path::new);
    match check_program_with_path(program, source.as_bytes(), path) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .into_iter()
            .map(|err| to_validation_error(err, source))
            .collect(),
    }
}

pub fn check_types_with_externals(
    program: &Program,
    source: &str,
    file_path: Option<&str>,
    externals: &std::collections::HashMap<String, ExternalFunctionSig>,
) -> Vec<ValidationError> {
    let path = file_path
        .filter(|path| !path.is_empty())
        .map(Path::new);
    match check_program_with_path_and_externals(program, source.as_bytes(), path, externals) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .into_iter()
            .map(|err| to_validation_error(err, source))
            .collect(),
    }
}

fn to_validation_error(error: PhpTypeError, source: &str) -> ValidationError {
    let (line, column, underline_length) = span_location(error.span, source);
    ValidationError {
        kind: ErrorKind::TypeError,
        line,
        column,
        message: error.message,
        help_text: "Fix the type mismatch or update the annotation.".to_string(),
        suggestion: None,
        underline_length,
        severity: Severity::Error,
    }
}

fn span_location(span: php_rs::parser::span::Span, source: &str) -> (usize, usize, usize) {
    if let Some(info) = span.line_info(source.as_bytes()) {
        let padding = std::cmp::min(info.line_text.len(), info.column.saturating_sub(1));
        let highlight_len = std::cmp::max(
            1,
            std::cmp::min(span.len(), info.line_text.len().saturating_sub(padding)),
        );
        (info.line, info.column, highlight_len)
    } else {
        (1, 1, 1)
    }
}
