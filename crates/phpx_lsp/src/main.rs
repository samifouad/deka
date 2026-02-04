use bumpalo::Bump;
use modules_php::compiler_api::compile_phpx;
use modules_php::validation::{
    format_validation_error, format_validation_warning, Severity, ValidationError,
    ValidationWarning,
};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticOptions, DiagnosticServerCapabilities, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
    InitializedParams, MessageType, Position, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    _client: Client,
}

impl Backend {
    async fn validate_document(&self, uri: Url, text: &str) {
        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.to_str().map(|path| path.to_string()))
            .unwrap_or_else(|| uri.to_string());

        let arena = Bump::new();
        let result = compile_phpx(text, &file_path, &arena);
        let mut diagnostics = Vec::new();

        for error in result.errors {
            diagnostics.push(diagnostic_from_error(&file_path, text, &error));
        }

        for warning in result.warnings {
            diagnostics.push(diagnostic_from_warning(&file_path, text, &warning));
        }

        self._client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        let diagnostics = DiagnosticOptions {
            identifier: Some("phpx".to_string()),
            ..DiagnosticOptions::default()
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(diagnostics)),
                ..ServerCapabilities::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self._client
            .log_message(MessageType::INFO, "PHPX LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self._client
            .log_message(
                MessageType::INFO,
                format!("Opened {}", params.text_document.uri),
            )
            .await;

        self.validate_document(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self._client
            .log_message(
                MessageType::INFO,
                format!("Changed {}", params.text_document.uri),
            )
            .await;

        let text = params
            .content_changes
            .last()
            .map(|change| change.text.as_str())
            .unwrap_or("");
        self.validate_document(params.text_document.uri, text).await;
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Backend { _client: client });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

fn diagnostic_from_error(file_path: &str, source: &str, error: &ValidationError) -> Diagnostic {
    Diagnostic {
        range: diagnostic_range(error.line, error.column, error.underline_length),
        severity: Some(severity_to_lsp(error.severity)),
        code: Some(tower_lsp::lsp_types::NumberOrString::String(
            error.kind.as_str().to_string(),
        )),
        source: Some("phpx".to_string()),
        message: format_validation_error(source, file_path, error),
        ..Diagnostic::default()
    }
}

fn diagnostic_from_warning(
    file_path: &str,
    source: &str,
    warning: &ValidationWarning,
) -> Diagnostic {
    Diagnostic {
        range: diagnostic_range(warning.line, warning.column, warning.underline_length),
        severity: Some(severity_to_lsp(warning.severity)),
        code: Some(tower_lsp::lsp_types::NumberOrString::String(
            warning.kind.as_str().to_string(),
        )),
        source: Some("phpx".to_string()),
        message: format_validation_warning(source, file_path, warning),
        ..Diagnostic::default()
    }
}

fn severity_to_lsp(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
    }
}

fn diagnostic_range(line: usize, column: usize, underline_length: usize) -> Range {
    let line = line.saturating_sub(1) as u32;
    let start_char = column.saturating_sub(1) as u32;
    let end_char = start_char + underline_length.max(1) as u32;
    Range {
        start: Position {
            line,
            character: start_char,
        },
        end: Position {
            line,
            character: end_char,
        },
    }
}
