pub mod imports;
pub mod exports;
pub mod syntax;
pub mod type_syntax;

use php_rs::parser::ast::Program;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ErrorKind {
    SyntaxError,
    UnexpectedToken,
    InvalidToken,
    TypeError,
    TypeMismatch,
    UnknownType,
    ImportError,
    ExportError,
    ModuleError,
    WasmError,
    NullNotAllowed,
    OopNotAllowed,
    NamespaceNotAllowed,
    JsxError,
    StructError,
    EnumError,
    PatternError,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::SyntaxError => "Syntax Error",
            ErrorKind::UnexpectedToken => "Unexpected Token",
            ErrorKind::InvalidToken => "Invalid Token",
            ErrorKind::TypeError => "Type Error",
            ErrorKind::TypeMismatch => "Type Mismatch",
            ErrorKind::UnknownType => "Unknown Type",
            ErrorKind::ImportError => "Import Error",
            ErrorKind::ExportError => "Export Error",
            ErrorKind::ModuleError => "Module Error",
            ErrorKind::WasmError => "WASM Error",
            ErrorKind::NullNotAllowed => "Null Not Allowed",
            ErrorKind::OopNotAllowed => "OOP Not Allowed",
            ErrorKind::NamespaceNotAllowed => "Namespace Not Allowed",
            ErrorKind::JsxError => "JSX Error",
            ErrorKind::StructError => "Struct Error",
            ErrorKind::EnumError => "Enum Error",
            ErrorKind::PatternError => "Pattern Error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub kind: ErrorKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub help_text: String,
    pub underline_length: usize,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationWarning {
    pub kind: ErrorKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub help_text: String,
    pub underline_length: usize,
    pub severity: Severity,
}

#[derive(Debug)]
pub struct ValidationResult<'a> {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub ast: Option<Program<'a>>,
}

pub fn parse_errors_to_validation_errors(
    source: &str,
    errors: &[php_rs::parser::ast::ParseError],
) -> Vec<ValidationError> {
    errors
        .iter()
        .map(|error| {
            let (line, column, underline_len) = if let Some(info) =
                error.span.line_info(source.as_bytes())
            {
                let padding = std::cmp::min(
                    info.line_text.len(),
                    info.column.saturating_sub(1),
                );
                let highlight_len = std::cmp::max(
                    1,
                    std::cmp::min(error.span.len(), info.line_text.len().saturating_sub(padding)),
                );
                (info.line, info.column, highlight_len)
            } else {
                (1, 1, 1)
            };

            ValidationError {
                kind: ErrorKind::from_parse_error(error),
                line,
                column,
                message: error.message.to_string(),
                help_text: error.help_text.to_string(),
                underline_length: underline_len,
                severity: Severity::Error,
            }
        })
        .collect()
}

impl ErrorKind {
    pub fn from_parse_error(error: &php_rs::parser::ast::ParseError) -> Self {
        match error.error_kind {
            "Syntax Error" => ErrorKind::SyntaxError,
            "Unexpected Token" => ErrorKind::UnexpectedToken,
            "Invalid Token" => ErrorKind::InvalidToken,
            _ => ErrorKind::SyntaxError,
        }
    }
}
