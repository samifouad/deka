use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NotFound,
    AlreadyExists,
    NotSupported,
    PermissionDenied,
    InvalidInput,
    Io,
    Busy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct WosixError {
    pub code: ErrorCode,
    pub message: String,
}

impl WosixError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for WosixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for WosixError {}

pub type Result<T> = std::result::Result<T, WosixError>;
