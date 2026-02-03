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
