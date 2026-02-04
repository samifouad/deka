use bumpalo::Bump;
use modules_php::compiler_api::compile_phpx;
use modules_php::validation::{
    format_validation_error, format_validation_warning, Severity, ValidationError,
    ValidationWarning,
};
use php_rs::parser::ast::{
    ClassKind, ClassMember, Expr, ExprId, Name, ObjectKey, Param, Program, Stmt, StmtId, Type,
};
use php_rs::parser::lexer::token::Token;
use php_rs::parser::span::Span;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticOptions, DiagnosticServerCapabilities, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, DocumentSymbol, DocumentSymbolParams, Hover, HoverContents,
    InitializeParams, InitializeResult, InitializedParams, Location, MarkupContent, MarkupKind,
    MessageType, OneOf, Position, Range, ReferenceParams, RenameParams, ServerCapabilities, SymbolKind, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

struct Backend {
    _client: Client,
    documents: Arc<RwLock<HashMap<Url, String>>>,
    workspace_roots: Arc<RwLock<Vec<PathBuf>>>,
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

    async fn get_document(&self, uri: &Url) -> Option<String> {
        let docs = self.documents.read().await;
        docs.get(uri).cloned()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        let mut roots = Vec::new();
        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                if let Ok(path) = folder.uri.to_file_path() {
                    roots.push(path);
                }
            }
        } else if let Some(root_uri) = params.root_uri {
            if let Ok(path) = root_uri.to_file_path() {
                roots.push(path);
            }
        }
        if roots.is_empty() {
            if let Ok(current) = std::env::current_dir() {
                roots.push(current);
            }
        }
        *self.workspace_roots.write().await = roots;

        let diagnostics = DiagnosticOptions {
            identifier: Some("phpx".to_string()),
            ..DiagnosticOptions::default()
        };

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(diagnostics)),
                hover_provider: Some(true.into()),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["'".to_string(), "\"".to_string(), ".".to_string(), "\\".to_string()]),
                    ..CompletionOptions::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
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

        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents.write().await.insert(uri.clone(), text.clone());
        self.validate_document(uri, &text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self._client
            .log_message(
                MessageType::INFO,
                format!("Changed {}", params.text_document.uri),
            )
            .await;

        let uri = params.text_document.uri;
        let text = params
            .content_changes
            .last()
            .map(|change| change.text.as_str())
            .unwrap_or("")
            .to_string();
        self.documents.write().await.insert(uri.clone(), text.clone());
        self.validate_document(uri, &text).await;
    }

    async fn hover(&self, params: tower_lsp::lsp_types::HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(text) = self.get_document(&uri).await else {
            return Ok(None);
        };

        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.to_str().map(|path| path.to_string()))
            .unwrap_or_else(|| uri.to_string());

        let line_index = LineIndex::new(&text);
        let offset = match line_index.position_to_offset(position) {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let mut hover_text = None;
        with_program(&text, &file_path, |program, source| {
            let index = build_index(program, source);
            hover_text = index.hover_at(offset);
        });

        if hover_text.is_none() {
            hover_text = hover_from_import(&text, offset);
        }
        if hover_text.is_none() {
            hover_text = hover_from_wasm_import(&text, &file_path, offset);
        }

        let Some(value) = hover_text else {
            return Ok(None);
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Some(text) = self.get_document(&uri).await else {
            return Ok(None);
        };
        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.to_str().map(|path| path.to_string()))
            .unwrap_or_else(|| uri.to_string());

        let line_index = LineIndex::new(&text);
        let offset = match line_index.position_to_offset(position) {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let mut dot_items = None;
        with_program(&text, &file_path, |program, source| {
            let index = build_index(program, source);
            dot_items = completion_for_dot(&index, source, offset);
        });
        if let Some(items) = dot_items {
            return Ok(Some(CompletionResponse::Array(items)));
        }

        if let Some(items) = completion_for_import(&text, &file_path, offset) {
            return Ok(Some(CompletionResponse::Array(items)));
        }

        let mut items = builtin_completion_items();
        items.extend(stdlib_completion_items());
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: tower_lsp::lsp_types::GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<tower_lsp::lsp_types::GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(text) = self.get_document(&uri).await else {
            return Ok(None);
        };
        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.to_str().map(|path| path.to_string()))
            .unwrap_or_else(|| uri.to_string());

        let line_index = LineIndex::new(&text);
        let offset = match line_index.position_to_offset(position) {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let mut location = None;
        with_program(&text, &file_path, |program, source| {
            let index = build_index(program, source);
            location = index.definition_at(offset, &uri, &line_index, source);
        });

        if let Some(loc) = location {
            return Ok(Some(tower_lsp::lsp_types::GotoDefinitionResponse::Scalar(loc)));
        }

        if let Some(word) = word_at_offset(text.as_bytes(), offset) {
            if let Some(loc) = definition_for_imported_symbol(&text, &file_path, &word) {
                return Ok(Some(tower_lsp::lsp_types::GotoDefinitionResponse::Scalar(
                    loc,
                )));
            }
        }

        if let Some(loc) = definition_for_import_module(&text, &file_path, offset) {
            return Ok(Some(tower_lsp::lsp_types::GotoDefinitionResponse::Scalar(loc)));
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> tower_lsp::jsonrpc::Result<Option<tower_lsp::lsp_types::DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let Some(text) = self.get_document(&uri).await else {
            return Ok(None);
        };
        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.to_str().map(|path| path.to_string()))
            .unwrap_or_else(|| uri.to_string());

        let line_index = LineIndex::new(&text);
        let mut symbols = Vec::new();
        with_program(&text, &file_path, |program, source| {
            let index = build_index(program, source);
            symbols = index.document_symbols(&line_index);
        });

        Ok(Some(tower_lsp::lsp_types::DocumentSymbolResponse::Nested(
            symbols,
        )))
    }

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let mut text = self.get_document(&uri).await;
        if text.is_none() {
            if let Ok(path) = uri.to_file_path() {
                text = fs::read_to_string(path).ok();
            }
        }
        let Some(text) = text else {
            return Ok(None);
        };
        let line_index = LineIndex::new(&text);
        let offset = match line_index.position_to_offset(position) {
            Some(offset) => offset,
            None => return Ok(None),
        };
        let Some(word) = word_at_offset(text.as_bytes(), offset) else {
            return Ok(None);
        };

        let mut roots = self.workspace_roots.read().await.clone();
        if roots.is_empty() {
            if let Ok(path) = uri.to_file_path() {
                if let Some(parent) = path.parent() {
                    roots.push(parent.to_path_buf());
                }
            }
        }

        let mut locations = Vec::new();
        for root in roots {
            for file in collect_phpx_files(&root) {
                let file_uri = match Url::from_file_path(&file) {
                    Ok(uri) => uri,
                    Err(_) => continue,
                };
                let content = if file_uri == uri {
                    text.clone()
                } else {
                    fs::read_to_string(&file).unwrap_or_default()
                };
                let line_index = LineIndex::new(&content);
                for span in find_word_occurrences(content.as_bytes(), &word) {
                    locations.push(Location {
                        uri: file_uri.clone(),
                        range: span_to_range(span, &line_index),
                    });
                }
            }
        }

        Ok(Some(locations))
    }

    async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        let mut text = self.get_document(&uri).await;
        if text.is_none() {
            if let Ok(path) = uri.to_file_path() {
                text = fs::read_to_string(path).ok();
            }
        }
        let Some(text) = text else {
            return Ok(None);
        };
        let line_index = LineIndex::new(&text);
        let offset = match line_index.position_to_offset(position) {
            Some(offset) => offset,
            None => return Ok(None),
        };
        let Some(word) = word_at_offset(text.as_bytes(), offset) else {
            return Ok(None);
        };

        let mut roots = self.workspace_roots.read().await.clone();
        if roots.is_empty() {
            if let Ok(path) = uri.to_file_path() {
                if let Some(parent) = path.parent() {
                    roots.push(parent.to_path_buf());
                }
            }
        }

        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        for root in roots {
            for file in collect_phpx_files(&root) {
                let file_uri = match Url::from_file_path(&file) {
                    Ok(uri) => uri,
                    Err(_) => continue,
                };
                let content = if file_uri == uri {
                    text.clone()
                } else {
                    fs::read_to_string(&file).unwrap_or_default()
                };
                let line_index = LineIndex::new(&content);
                let mut edits = Vec::new();
                for span in find_word_occurrences(content.as_bytes(), &word) {
                    edits.push(TextEdit {
                        range: span_to_range(span, &line_index),
                        new_text: new_name.clone(),
                    });
                }
                if !edits.is_empty() {
                    changes.insert(file_uri, edits);
                }
            }
        }

        if changes.is_empty() {
            return Ok(None);
        }

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Backend {
        _client: client,
        documents: Arc::new(RwLock::new(HashMap::new())),
        workspace_roots: Arc::new(RwLock::new(Vec::new())),
    });
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

struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0usize];
        for (idx, byte) in source.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self { line_starts }
    }

    fn offset_to_position(&self, offset: usize) -> Position {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        Position {
            line: line as u32,
            character: offset.saturating_sub(line_start) as u32,
        }
    }

    fn position_to_offset(&self, position: Position) -> Option<usize> {
        let line = position.line as usize;
        let line_start = *self.line_starts.get(line)?;
        Some(line_start + position.character as usize)
    }
}

fn with_program<F, R>(source: &str, file_path: &str, f: F) -> Option<R>
where
    F: FnOnce(&Program, &[u8]) -> R,
{
    let arena = Bump::new();
    let result = compile_phpx(source, file_path, &arena);
    let program = result.ast?;
    Some(f(&program, source.as_bytes()))
}

#[derive(Default)]
struct SymbolIndex {
    functions: Vec<FunctionInfo>,
    structs: Vec<StructInfo>,
    enums: Vec<EnumInfo>,
    type_aliases: Vec<TypeAliasInfo>,
    globals: Vec<VarInfo>,
    consts: Vec<ConstInfo>,
}

#[derive(Clone)]
struct FunctionInfo {
    name: String,
    span: Span,
    signature: String,
    vars: Vec<VarInfo>,
    scope_span: Span,
}

#[derive(Clone)]
struct VarInfo {
    name: String,
    span: Span,
    ty: Option<String>,
}

#[derive(Clone)]
struct StructInfo {
    name: String,
    span: Span,
    fields: Vec<FieldInfo>,
}

#[derive(Clone)]
struct FieldInfo {
    name: String,
    span: Span,
    ty: Option<String>,
}

#[derive(Clone)]
struct EnumInfo {
    name: String,
    span: Span,
    cases: Vec<EnumCaseInfo>,
}

#[derive(Clone)]
struct EnumCaseInfo {
    name: String,
    span: Span,
}

#[derive(Clone)]
struct TypeAliasInfo {
    name: String,
    span: Span,
    ty: String,
}

#[derive(Clone)]
struct ConstInfo {
    name: String,
    span: Span,
}

impl SymbolIndex {
    fn hover_at(&self, offset: usize) -> Option<String> {
        for func in &self.functions {
            if span_contains(func.span, offset) {
                return Some(format!("```php\n{}\n```", func.signature));
            }
            for var in &func.vars {
                if span_contains(var.span, offset) {
                    let ty = var.ty.clone().unwrap_or_else(|| "unknown".to_string());
                    return Some(format!("```php\n{}: {}\n```", var.name, ty));
                }
            }
        }

        for strukt in &self.structs {
            if span_contains(strukt.span, offset) {
                let mut out = format!("```php\nstruct {}\n", strukt.name);
                for field in &strukt.fields {
                    let ty = field.ty.clone().unwrap_or_else(|| "mixed".to_string());
                    out.push_str(&format!("  ${}: {}\n", field.name.trim_start_matches('$'), ty));
                }
                out.push_str("```");
                return Some(out);
            }
            for field in &strukt.fields {
                if span_contains(field.span, offset) {
                    let ty = field.ty.clone().unwrap_or_else(|| "mixed".to_string());
                    return Some(format!("```php\n${}: {}\n```", field.name.trim_start_matches('$'), ty));
                }
            }
        }

        for en in &self.enums {
            if span_contains(en.span, offset) {
                let mut out = format!("```php\nenum {}\n", en.name);
                for case_info in &en.cases {
                    out.push_str(&format!("  case {}\n", case_info.name));
                }
                out.push_str("```");
                return Some(out);
            }
            for case_info in &en.cases {
                if span_contains(case_info.span, offset) {
                    return Some(format!("```php\n{}::{}\n```", en.name, case_info.name));
                }
            }
        }

        for alias in &self.type_aliases {
            if span_contains(alias.span, offset) {
                return Some(format!("```php\ntype {} = {}\n```", alias.name, alias.ty));
            }
        }

        for konst in &self.consts {
            if span_contains(konst.span, offset) {
                return Some(format!("```php\nconst {}\n```", konst.name));
            }
        }

        None
    }

    fn definition_at(
        &self,
        offset: usize,
        uri: &Url,
        line_index: &LineIndex,
        source: &[u8],
    ) -> Option<Location> {
        let word = word_at_offset(source, offset);
        let Some(word) = word else {
            return None;
        };

        if word.starts_with('$') {
            for func in &self.functions {
                if span_contains(func.scope_span, offset) {
                    let mut best: Option<&VarInfo> = None;
                    for var in &func.vars {
                        if var.name == word && var.span.start <= offset {
                            if best.is_none() || var.span.start > best.unwrap().span.start {
                                best = Some(var);
                            }
                        }
                    }
                    if let Some(var) = best {
                        return Some(Location {
                            uri: uri.clone(),
                            range: span_to_range(var.span, line_index),
                        });
                    }
                    return None;
                }
            }
            let mut best: Option<&VarInfo> = None;
            for var in &self.globals {
                if var.name == word && var.span.start <= offset {
                    if best.is_none() || var.span.start > best.unwrap().span.start {
                        best = Some(var);
                    }
                }
            }
            if let Some(var) = best {
                return Some(Location {
                    uri: uri.clone(),
                    range: span_to_range(var.span, line_index),
                });
            }
        }

        for func in &self.functions {
            if func.name == word {
                return Some(Location {
                    uri: uri.clone(),
                    range: span_to_range(func.span, line_index),
                });
            }
        }
        for strukt in &self.structs {
            if strukt.name == word {
                return Some(Location {
                    uri: uri.clone(),
                    range: span_to_range(strukt.span, line_index),
                });
            }
        }
        for en in &self.enums {
            if en.name == word {
                return Some(Location {
                    uri: uri.clone(),
                    range: span_to_range(en.span, line_index),
                });
            }
        }
        for alias in &self.type_aliases {
            if alias.name == word {
                return Some(Location {
                    uri: uri.clone(),
                    range: span_to_range(alias.span, line_index),
                });
            }
        }
        for konst in &self.consts {
            if konst.name == word {
                return Some(Location {
                    uri: uri.clone(),
                    range: span_to_range(konst.span, line_index),
                });
            }
        }
        None
    }

    fn document_symbols(&self, line_index: &LineIndex) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        for func in &self.functions {
            symbols.push(DocumentSymbol {
                name: func.name.clone(),
                detail: Some(func.signature.clone()),
                kind: SymbolKind::FUNCTION,
                range: span_to_range(func.scope_span, line_index),
                selection_range: span_to_range(func.span, line_index),
                children: None,
                deprecated: None,
                tags: None,
            });
        }
        for strukt in &self.structs {
            let fields = strukt
                .fields
                .iter()
                .map(|field| DocumentSymbol {
                    name: field.name.trim_start_matches('$').to_string(),
                    detail: field.ty.clone(),
                    kind: SymbolKind::FIELD,
                    range: span_to_range(field.span, line_index),
                    selection_range: span_to_range(field.span, line_index),
                    children: None,
                    deprecated: None,
                    tags: None,
                })
                .collect();
            symbols.push(DocumentSymbol {
                name: strukt.name.clone(),
                detail: Some("struct".to_string()),
                kind: SymbolKind::STRUCT,
                range: span_to_range(strukt.span, line_index),
                selection_range: span_to_range(strukt.span, line_index),
                children: Some(fields),
                deprecated: None,
                tags: None,
            });
        }
        for en in &self.enums {
            let cases = en
                .cases
                .iter()
                .map(|case_info| DocumentSymbol {
                    name: case_info.name.clone(),
                    detail: Some("case".to_string()),
                    kind: SymbolKind::ENUM_MEMBER,
                    range: span_to_range(case_info.span, line_index),
                    selection_range: span_to_range(case_info.span, line_index),
                    children: None,
                    deprecated: None,
                    tags: None,
                })
                .collect();
            symbols.push(DocumentSymbol {
                name: en.name.clone(),
                detail: Some("enum".to_string()),
                kind: SymbolKind::ENUM,
                range: span_to_range(en.span, line_index),
                selection_range: span_to_range(en.span, line_index),
                children: Some(cases),
                deprecated: None,
                tags: None,
            });
        }
        for alias in &self.type_aliases {
            symbols.push(DocumentSymbol {
                name: alias.name.clone(),
                detail: Some(alias.ty.clone()),
                kind: SymbolKind::TYPE_PARAMETER,
                range: span_to_range(alias.span, line_index),
                selection_range: span_to_range(alias.span, line_index),
                children: None,
                deprecated: None,
                tags: None,
            });
        }
        for konst in &self.consts {
            symbols.push(DocumentSymbol {
                name: konst.name.clone(),
                detail: Some("const".to_string()),
                kind: SymbolKind::CONSTANT,
                range: span_to_range(konst.span, line_index),
                selection_range: span_to_range(konst.span, line_index),
                children: None,
                deprecated: None,
                tags: None,
            });
        }
        symbols
    }

    fn var_type_at(&self, offset: usize, name: &str) -> Option<String> {
        for func in &self.functions {
            if span_contains(func.scope_span, offset) {
                let mut best: Option<&VarInfo> = None;
                for var in &func.vars {
                    if var.name == name && var.span.start <= offset {
                        if best.is_none() || var.span.start > best.unwrap().span.start {
                            best = Some(var);
                        }
                    }
                }
                return best.and_then(|var| var.ty.clone());
            }
        }
        let mut best: Option<&VarInfo> = None;
        for var in &self.globals {
            if var.name == name && var.span.start <= offset {
                if best.is_none() || var.span.start > best.unwrap().span.start {
                    best = Some(var);
                }
            }
        }
        best.and_then(|var| var.ty.clone())
    }

    fn fields_for_type(&self, ty: &str) -> Option<Vec<String>> {
        if let Some(strukt) = self.structs.iter().find(|s| s.name == ty) {
            return Some(
                strukt
                    .fields
                    .iter()
                    .map(|field| field.name.trim_start_matches('$').to_string())
                    .collect(),
            );
        }
        if let Some(alias) = self.type_aliases.iter().find(|alias| alias.name == ty) {
            return self.fields_for_type(alias.ty.trim());
        }
        if let Some(inner) = ty.strip_prefix("Option<").and_then(|value| value.strip_suffix('>')) {
            return self.fields_for_type(inner.trim());
        }
        if let Some(inner) = ty.strip_prefix("Result<").and_then(|value| value.strip_suffix('>')) {
            let first = inner.split(',').next().unwrap_or(inner).trim();
            return self.fields_for_type(first);
        }
        if let Some(inner) = ty.strip_prefix("Object<{").and_then(|value| value.strip_suffix("}>")) {
            let mut fields = Vec::new();
            for part in inner.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let name = part
                    .split(':')
                    .next()
                    .unwrap_or(part)
                    .trim()
                    .trim_end_matches('?');
                if !name.is_empty() {
                    fields.push(name.to_string());
                }
            }
            return Some(fields);
        }
        None
    }
}

fn build_index(program: &Program, source: &[u8]) -> SymbolIndex {
    let mut index = SymbolIndex::default();
    for stmt in program.statements.iter() {
        collect_stmt(stmt, source, &mut index);
    }
    index
}

fn collect_stmt(stmt: StmtId, source: &[u8], index: &mut SymbolIndex) {
    match stmt {
        Stmt::Expression { expr, .. } => {
            collect_vars_in_expr(expr, source, &mut index.globals);
        }
        Stmt::Return { expr: Some(expr), .. } => {
            collect_vars_in_expr(expr, source, &mut index.globals);
        }
        Stmt::Function {
            name,
            params,
            return_type,
            body,
            span,
            ..
        } => {
            let fn_name = token_text(source, name);
            let signature =
                format_function_signature(fn_name.as_str(), params, *return_type, source);
            let mut vars = Vec::new();
            for param in *params {
                let param_name = token_text(source, param.name);
                let ty = param.ty.map(|ty| format_type(ty, source));
                vars.push(VarInfo {
                    name: param_name,
                    span: param.name.span,
                    ty,
                });
            }
            collect_vars_in_block(body, source, &mut vars);
            index.functions.push(FunctionInfo {
                name: fn_name,
                span: name.span,
                signature,
                vars,
                scope_span: *span,
            });
        }
        Stmt::Class {
            kind,
            name,
            members,
            span: _,
            ..
        } => {
            if *kind == ClassKind::Struct {
                let struct_name = token_text(source, name);
                let mut fields = Vec::new();
                for member in *members {
                    if let ClassMember::Property { ty, entries, .. } = member {
                        for entry in *entries {
                            let field_name = token_text(source, entry.name);
                            let field_ty = ty.map(|ty| format_type(ty, source));
                            fields.push(FieldInfo {
                                name: field_name,
                                span: entry.name.span,
                                ty: field_ty,
                            });
                        }
                    }
                }
                index.structs.push(StructInfo {
                    name: struct_name,
                    span: name.span,
                    fields,
                });
            }
        }
        Stmt::Enum { name, members, span: _, .. } => {
            let enum_name = token_text(source, name);
            let mut cases = Vec::new();
            for member in *members {
                if let ClassMember::Case { name, .. } = member {
                    cases.push(EnumCaseInfo {
                        name: token_text(source, name),
                        span: name.span,
                    });
                }
            }
            index.enums.push(EnumInfo {
                name: enum_name,
                span: name.span,
                cases,
            });
        }
        Stmt::TypeAlias { name, ty, .. } => {
            let alias_name = token_text(source, name);
            let alias_ty = format_type(ty, source);
            index.type_aliases.push(TypeAliasInfo {
                name: alias_name,
                span: name.span,
                ty: alias_ty,
            });
        }
        Stmt::Const { consts, .. } => {
            for konst in *consts {
                let const_name = token_text(source, konst.name);
                index.consts.push(ConstInfo {
                    name: const_name,
                    span: konst.name.span,
                });
            }
        }
        Stmt::Block { statements, .. } => {
            for stmt in *statements {
                collect_stmt(stmt, source, index);
            }
        }
        Stmt::If {
            then_block,
            else_block,
            ..
        } => {
            for stmt in *then_block {
                collect_stmt(stmt, source, index);
            }
            if let Some(block) = else_block {
                for stmt in *block {
                    collect_stmt(stmt, source, index);
                }
            }
        }
        Stmt::While { body, .. }
        | Stmt::DoWhile { body, .. }
        | Stmt::For { body, .. }
        | Stmt::Foreach { body, .. } => {
            for stmt in *body {
                collect_stmt(stmt, source, index);
            }
        }
        _ => {}
    }
}

fn collect_vars_in_block(body: &[StmtId], source: &[u8], vars: &mut Vec<VarInfo>) {
    for stmt in body {
        match stmt {
            Stmt::Expression { expr, .. } => collect_vars_in_expr(expr, source, vars),
            Stmt::Return { expr: Some(expr), .. } => collect_vars_in_expr(expr, source, vars),
            Stmt::If { then_block, else_block, .. } => {
                collect_vars_in_block(then_block, source, vars);
                if let Some(block) = else_block {
                    collect_vars_in_block(block, source, vars);
                }
            }
            Stmt::While { body, .. }
            | Stmt::DoWhile { body, .. }
            | Stmt::For { body, .. }
            | Stmt::Foreach { body, .. }
            | Stmt::Block { statements: body, .. } => {
                collect_vars_in_block(body, source, vars);
            }
            _ => {}
        }
    }
}

fn collect_vars_in_expr(expr: ExprId, source: &[u8], vars: &mut Vec<VarInfo>) {
    match expr {
        Expr::Assign { var, expr, .. }
        | Expr::AssignOp { var, expr, .. }
        | Expr::AssignRef { var, expr, .. } => {
            if let Some((name, span)) = variable_name(var, source) {
                let ty = infer_expr_type(expr, source);
                vars.push(VarInfo { name, span, ty });
            }
            collect_vars_in_expr(expr, source, vars);
        }
        Expr::Binary { left, right, .. } => {
            collect_vars_in_expr(left, source, vars);
            collect_vars_in_expr(right, source, vars);
        }
        Expr::Unary { expr, .. } => collect_vars_in_expr(expr, source, vars),
        Expr::Call { func, args, .. } => {
            collect_vars_in_expr(func, source, vars);
            for arg in *args {
                collect_vars_in_expr(arg.value, source, vars);
            }
        }
        Expr::Array { items, .. } => {
            for item in *items {
                collect_vars_in_expr(item.value, source, vars);
            }
        }
        Expr::ObjectLiteral { items, .. } => {
            for item in *items {
                collect_vars_in_expr(item.value, source, vars);
            }
        }
        Expr::StructLiteral { fields, .. } => {
            for field in *fields {
                collect_vars_in_expr(field.value, source, vars);
            }
        }
        Expr::JsxElement { children, .. } | Expr::JsxFragment { children, .. } => {
            for child in *children {
                if let php_rs::parser::ast::JsxChild::Expr(expr) = child {
                    collect_vars_in_expr(expr, source, vars);
                }
            }
        }
        Expr::ArrayDimFetch { array, dim, .. } => {
            collect_vars_in_expr(array, source, vars);
            if let Some(dim) = dim {
                collect_vars_in_expr(dim, source, vars);
            }
        }
        Expr::PropertyFetch { target, property, .. } => {
            collect_vars_in_expr(target, source, vars);
            collect_vars_in_expr(property, source, vars);
        }
        Expr::MethodCall { target, method, args, .. } => {
            collect_vars_in_expr(target, source, vars);
            collect_vars_in_expr(method, source, vars);
            for arg in *args {
                collect_vars_in_expr(arg.value, source, vars);
            }
        }
        Expr::StaticCall { class, method, args, .. } => {
            collect_vars_in_expr(class, source, vars);
            collect_vars_in_expr(method, source, vars);
            for arg in *args {
                collect_vars_in_expr(arg.value, source, vars);
            }
        }
        Expr::ClassConstFetch { class, constant, .. } => {
            collect_vars_in_expr(class, source, vars);
            collect_vars_in_expr(constant, source, vars);
        }
        Expr::New { class, args, .. } => {
            collect_vars_in_expr(class, source, vars);
            for arg in *args {
                collect_vars_in_expr(arg.value, source, vars);
            }
        }
        Expr::Ternary { condition, if_true, if_false, .. } => {
            collect_vars_in_expr(condition, source, vars);
            if let Some(expr) = if_true {
                collect_vars_in_expr(expr, source, vars);
            }
            collect_vars_in_expr(if_false, source, vars);
        }
        Expr::Include { expr, .. } => collect_vars_in_expr(expr, source, vars),
        Expr::DotAccess { target, .. } => collect_vars_in_expr(target, source, vars),
        Expr::PostInc { var, .. } | Expr::PostDec { var, .. } => {
            collect_vars_in_expr(var, source, vars)
        }
        Expr::InterpolatedString { parts, .. } | Expr::ShellExec { parts, .. } => {
            for part in *parts {
                collect_vars_in_expr(part, source, vars);
            }
        }
        _ => {}
    }
}

fn variable_name(expr: ExprId, source: &[u8]) -> Option<(String, Span)> {
    match expr {
        Expr::Variable { name, .. } => {
            let text = span_text(source, name);
            Some((text, *name))
        }
        _ => None,
    }
}

fn infer_expr_type(expr: ExprId, source: &[u8]) -> Option<String> {
    match expr {
        Expr::Integer { .. } => Some("int".to_string()),
        Expr::Float { .. } => Some("float".to_string()),
        Expr::Boolean { .. } => Some("bool".to_string()),
        Expr::String { .. } | Expr::InterpolatedString { .. } => Some("string".to_string()),
        Expr::Array { .. } => Some("array".to_string()),
        Expr::ObjectLiteral { items, .. } => {
            let mut fields = Vec::new();
            for item in *items {
                let key = match item.key {
                    ObjectKey::Ident(token) | ObjectKey::String(token) => token_text(source, token),
                };
                let key = key.trim_matches('"').trim_matches('\'').to_string();
                if !key.is_empty() {
                    fields.push(format!("{}: mixed", key));
                }
            }
            if fields.is_empty() {
                Some("Object".to_string())
            } else {
                Some(format!("Object<{{{}}}>", fields.join(", ")))
            }
        }
        Expr::StructLiteral { name, .. } => Some(name_text(source, name)),
        Expr::JsxElement { .. } | Expr::JsxFragment { .. } => Some("VNode".to_string()),
        Expr::Null { .. } => Some("null".to_string()),
        _ => None,
    }
}

fn format_function_signature(
    name: &str,
    params: &[Param],
    return_type: Option<&Type>,
    source: &[u8],
) -> String {
    let mut sig = format!("function {}(", name);
    let mut first = true;
    for param in params {
        if !first {
            sig.push_str(", ");
        }
        first = false;
        if let Some(ty) = param.ty {
            sig.push_str(&format_type(ty, source));
            sig.push(' ');
        }
        sig.push_str(&token_text(source, param.name));
    }
    sig.push(')');
    if let Some(ty) = return_type {
        sig.push_str(": ");
        sig.push_str(&format_type(ty, source));
    }
    sig
}

fn format_type(ty: &Type, source: &[u8]) -> String {
    match ty {
        Type::Simple(token) => token_text(source, token),
        Type::Name(name) => name_text(source, name),
        Type::Union(types) => types
            .iter()
            .map(|ty| format_type(ty, source))
            .collect::<Vec<_>>()
            .join(" | "),
        Type::Intersection(types) => types
            .iter()
            .map(|ty| format_type(ty, source))
            .collect::<Vec<_>>()
            .join(" & "),
        Type::Nullable(inner) => format!("?{}", format_type(inner, source)),
        Type::ObjectShape(fields) => {
            let mut out = String::from("Object<{");
            let mut first = true;
            for field in *fields {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&token_text(source, field.name));
                if field.optional {
                    out.push('?');
                }
                out.push_str(": ");
                out.push_str(&format_type(field.ty, source));
            }
            out.push_str("}>");
            out
        }
        Type::Applied { base, args } => {
            let mut out = format_type(base, source);
            let rendered = args
                .iter()
                .map(|ty| format_type(ty, source))
                .collect::<Vec<_>>();
            out.push('<');
            out.push_str(&rendered.join(", "));
            out.push('>');
            out
        }
    }
}

fn name_text(source: &[u8], name: &Name) -> String {
    String::from_utf8_lossy(name.span.as_str(source)).to_string()
}

fn token_text(source: &[u8], token: &Token) -> String {
    String::from_utf8_lossy(token.text(source)).to_string()
}

fn span_text(source: &[u8], span: &Span) -> String {
    String::from_utf8_lossy(span.as_str(source)).to_string()
}

fn span_contains(span: Span, offset: usize) -> bool {
    span.start <= offset && offset < span.end
}

fn span_to_range(span: Span, line_index: &LineIndex) -> Range {
    Range {
        start: line_index.offset_to_position(span.start),
        end: line_index.offset_to_position(span.end),
    }
}

fn word_at_offset(source: &[u8], offset: usize) -> Option<String> {
    let bytes = source;
    if offset >= bytes.len() {
        return None;
    }
    let mut start = offset;
    let mut end = offset;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    let word = &bytes[start..end];
    if word.iter().all(|b| b.is_ascii_whitespace()) {
        return None;
    }
    Some(String::from_utf8_lossy(word).to_string())
}

fn word_before_dot(source: &[u8], offset: usize) -> Option<String> {
    if offset == 0 || source.get(offset - 1) != Some(&b'.') {
        return None;
    }
    if offset < 2 || !is_ident_char(source[offset - 2]) {
        return None;
    }
    let mut start = offset - 2;
    while start > 0 && is_ident_char(source[start - 1]) {
        start -= 1;
    }
    let end = offset - 1;
    if start >= end {
        return None;
    }
    Some(String::from_utf8_lossy(&source[start..end]).to_string())
}

fn is_ident_char(byte: u8) -> bool {
    byte == b'$'
        || byte == b'_'
        || (byte >= b'0' && byte <= b'9')
        || (byte >= b'a' && byte <= b'z')
        || (byte >= b'A' && byte <= b'Z')
        || byte == b'\\'
}

fn hover_from_import(source: &str, offset: usize) -> Option<String> {
    let imports = parse_imports(source);
    for import in imports {
        if import.span.start <= offset && offset < import.span.end {
            return Some(format!(
                "```php\nimport {} from '{}'\n```",
                import.local, import.from
            ));
        }
    }
    None
}

fn hover_from_wasm_import(source: &str, file_path: &str, offset: usize) -> Option<String> {
    let word = word_at_offset(source.as_bytes(), offset)?;
    let imports = parse_imports(source);
    for import in imports {
        if !import.is_wasm || import.local != word {
            continue;
        }
        let root = find_php_modules_root(Path::new(file_path))?;
        let path = resolve_module_file(&root, &import.from, true)?;
        let module_source = fs::read_to_string(path).ok()?;
        if let Some(signature) = wasm_stub_signature(&module_source, &import.imported) {
            return Some(format!("```php\n{}\n```", signature));
        }
    }
    None
}

fn definition_for_import_module(
    source: &str,
    file_path: &str,
    offset: usize,
) -> Option<Location> {
    let imports = parse_imports(source);
    for import in imports {
        if let Some(module_span) = import.module_span {
            if module_span.start <= offset && offset < module_span.end {
                let root = find_php_modules_root(Path::new(file_path))?;
                let path = resolve_module_file(&root, &import.from, import.is_wasm)?;
                let uri = Url::from_file_path(path).ok()?;
                return Some(Location {
                    uri,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                });
            }
        }
    }
    None
}

fn definition_for_imported_symbol(
    source: &str,
    file_path: &str,
    symbol: &str,
) -> Option<Location> {
    let imports = parse_imports(source);
    for import in imports {
        if import.local != symbol {
            continue;
        }
        let root = find_php_modules_root(Path::new(file_path))?;
        let path = resolve_module_file(&root, &import.from, import.is_wasm)?;
        let module_source = fs::read_to_string(&path).ok()?;
        let range = export_range_for_symbol(&module_source, &import.imported)
            .unwrap_or(Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            });
        let uri = Url::from_file_path(path).ok()?;
        return Some(Location { uri, range });
    }
    None
}

fn export_range_for_symbol(source: &str, symbol: &str) -> Option<Range> {
    let line_index = LineIndex::new(source);
    let mut offset = 0usize;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("export ") {
            offset += line.len() + 1;
            continue;
        }
        let rest = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some(open) = rest.find('{') {
            let close = rest.rfind('}').unwrap_or(rest.len());
            let inner = &rest[open + 1..close];
            for spec in inner.split(',') {
                let spec = spec.trim();
                if spec.is_empty() {
                    continue;
                }
                let (imported, local) = if let Some((left, right)) = spec.split_once(" as ") {
                    (left.trim(), right.trim())
                } else {
                    (spec, spec)
                };
                let name = if symbol == local { local } else if symbol == imported { imported } else { continue };
                if let Some(col) = line.find(name) {
                    let span = Span::new(offset + col, offset + col + name.len());
                    return Some(span_to_range(span, &line_index));
                }
            }
            offset += line.len() + 1;
            continue;
        }
        let keywords = ["function ", "const ", "type ", "struct ", "enum "];
        for keyword in keywords {
            if let Some(name) = rest.strip_prefix(keyword) {
                if let Some(token) = name.split_whitespace().next() {
                    if token == symbol {
                        if let Some(col) = line.find(token) {
                            let span = Span::new(offset + col, offset + col + token.len());
                            return Some(span_to_range(span, &line_index));
                        }
                    }
                }
            }
        }
        offset += line.len() + 1;
    }
    None
}

fn wasm_stub_signature(source: &str, symbol: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        let rest = match trimmed.strip_prefix("export function ") {
            Some(rest) => rest,
            None => continue,
        };
        let sig = rest.trim_end_matches(';').trim();
        let name = sig.split('(').next().unwrap_or("").trim();
        if name == symbol {
            return Some(format!("function {}", sig));
        }
    }
    None
}

struct ImportInfo {
    imported: String,
    local: String,
    from: String,
    span: Span,
    module_span: Option<Span>,
    is_wasm: bool,
}

fn parse_imports(source: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let mut offset = 0usize;
    for line in source.lines() {
        let line_len = line.len();
        let trimmed = line.trim_start();
        if trimmed.starts_with("import ") {
            if let (Some(open), Some(close)) = (line.find('{'), line.find('}')) {
                let spec_part = &line[open + 1..close];
                let module_info = parse_module_path_with_span(line, offset);
                let is_wasm = line.contains(" as wasm");
                let mut cursor = open + 1;
                for spec in spec_part.split(',') {
                    let spec_trim = spec.trim();
                    if spec_trim.is_empty() {
                        cursor += spec.len() + 1;
                        continue;
                    }
                    let (imported, local) = if let Some((left, right)) = spec_trim.split_once(" as ") {
                        (left.trim(), right.trim())
                    } else {
                        (spec_trim, spec_trim)
                    };
                    let local_pos = line[cursor..]
                        .find(local)
                        .map(|idx| cursor + idx);
                    if let Some(local_pos) = local_pos {
                        let span = Span::new(
                            offset + local_pos,
                            offset + local_pos + local.len(),
                        );
                        if let Some((module, module_span)) = module_info.clone() {
                            imports.push(ImportInfo {
                                imported: imported.to_string(),
                                local: local.to_string(),
                                from: module,
                                span,
                                module_span: Some(module_span),
                                is_wasm,
                            });
                        } else {
                            imports.push(ImportInfo {
                                imported: imported.to_string(),
                                local: local.to_string(),
                                from: imported.to_string(),
                                span,
                                module_span: None,
                                is_wasm,
                            });
                        }
                    }
                    cursor += spec.len() + 1;
                }
            }
        }
        offset += line_len + 1;
    }
    imports
}

fn parse_module_path(line: &str) -> Option<String> {
    let from_idx = line.find("from")?;
    let rest = &line[from_idx + 4..];
    let quote = rest.find(&['\'', '"'][..])?;
    let quote_char = rest.chars().nth(quote)?;
    let after = &rest[quote + 1..];
    let end = after.find(quote_char)?;
    Some(after[..end].to_string())
}

fn parse_module_path_with_span(line: &str, line_offset: usize) -> Option<(String, Span)> {
    let from_idx = line.find("from")?;
    let rest = &line[from_idx + 4..];
    let quote = rest.find(&['\'', '"'][..])?;
    let quote_char = rest.chars().nth(quote)?;
    let after = &rest[quote + 1..];
    let end = after.find(quote_char)?;
    let start = line_offset + from_idx + 4 + quote + 1;
    let end_pos = start + end;
    Some((after[..end].to_string(), Span::new(start, end_pos)))
}

struct ExportInfo {
    name: String,
    kind: Option<CompletionItemKind>,
}

fn module_exports(root: &Path, module_spec: &str, is_wasm: bool) -> Option<Vec<ExportInfo>> {
    let path = resolve_module_file(root, module_spec, is_wasm)?;
    let source = fs::read_to_string(path).ok()?;
    Some(parse_exported_names(&source))
}

fn parse_exported_names(source: &str) -> Vec<ExportInfo> {
    let mut exports = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("export ") {
            continue;
        }
        let rest = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some(open) = rest.find('{') {
            let close = rest.rfind('}').unwrap_or(rest.len());
            let inner = &rest[open + 1..close];
            for spec in inner.split(',') {
                let spec = spec.trim();
                if spec.is_empty() {
                    continue;
                }
                let name = if let Some((_, alias)) = spec.split_once(" as ") {
                    alias.trim()
                } else {
                    spec
                };
                if !name.is_empty() {
                    exports.push(ExportInfo {
                        name: name.to_string(),
                        kind: Some(CompletionItemKind::VARIABLE),
                    });
                }
            }
            continue;
        }
        if let Some(name) = rest.strip_prefix("function ") {
            if let Some(token) = name.split_whitespace().next() {
                exports.push(ExportInfo {
                    name: token.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                });
            }
            continue;
        }
        if let Some(name) = rest.strip_prefix("const ") {
            if let Some(token) = name.split_whitespace().next() {
                exports.push(ExportInfo {
                    name: token.to_string(),
                    kind: Some(CompletionItemKind::CONSTANT),
                });
            }
            continue;
        }
        if let Some(name) = rest.strip_prefix("type ") {
            if let Some(token) = name.split_whitespace().next() {
                exports.push(ExportInfo {
                    name: token.to_string(),
                    kind: Some(CompletionItemKind::TYPE_PARAMETER),
                });
            }
            continue;
        }
        if let Some(name) = rest.strip_prefix("struct ") {
            if let Some(token) = name.split_whitespace().next() {
                exports.push(ExportInfo {
                    name: token.to_string(),
                    kind: Some(CompletionItemKind::STRUCT),
                });
            }
            continue;
        }
        if let Some(name) = rest.strip_prefix("enum ") {
            if let Some(token) = name.split_whitespace().next() {
                exports.push(ExportInfo {
                    name: token.to_string(),
                    kind: Some(CompletionItemKind::ENUM),
                });
            }
            continue;
        }
    }
    exports
}

fn completion_for_import(source: &str, file_path: &str, offset: usize) -> Option<Vec<CompletionItem>> {
    let line_index = LineIndex::new(source);
    let position = line_index.offset_to_position(offset);
    let line = position.line as usize;
    let line_start = *line_index.line_starts.get(line)?;
    let line_end = source[line_start..]
        .find('\n')
        .map(|idx| line_start + idx)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    if !line_text.contains("import") || !line_text.contains("from") {
        return None;
    }
    let rel = offset.saturating_sub(line_start);

    if let (Some(open), Some(close)) = (line_text.find('{'), line_text.find('}')) {
        if rel > open && rel <= close {
            let module_spec = parse_module_path(line_text)?;
            let root = find_php_modules_root(Path::new(file_path))?;
            let is_wasm = line_text.contains(" as wasm");
            let exports = module_exports(&root, &module_spec, is_wasm)?;
            let prefix_start = line_text[..rel].rfind(',').map(|idx| idx + 1).unwrap_or(open + 1);
            let raw_prefix = line_text[prefix_start..rel].trim();
            let prefix = raw_prefix
                .split_whitespace()
                .last()
                .unwrap_or(raw_prefix)
                .trim();
            let mut items = Vec::new();
            for export in exports {
                if !prefix.is_empty() && !export.name.starts_with(prefix) {
                    continue;
                }
                items.push(CompletionItem {
                    label: export.name,
                    kind: export.kind,
                    ..CompletionItem::default()
                });
            }
            return Some(items);
        }
    }

    let before_cursor = &source[line_start..offset.min(source.len())];
    let quote_pos = before_cursor.rfind(&['\'', '"'][..])?;
    let prefix = &before_cursor[quote_pos + 1..];

    let root = find_php_modules_root(Path::new(file_path))?;
    let modules = list_php_modules(&root);
    let mut items = Vec::new();
    for module in modules {
        if !prefix.is_empty() && !module.starts_with(prefix) {
            continue;
        }
        items.push(CompletionItem {
            label: module.clone(),
            kind: Some(CompletionItemKind::MODULE),
            ..CompletionItem::default()
        });
    }
    Some(items)
}

fn completion_for_dot(
    index: &SymbolIndex,
    source: &[u8],
    offset: usize,
) -> Option<Vec<CompletionItem>> {
    if offset == 0 || source.get(offset - 1) != Some(&b'.') {
        return None;
    }
    let name = word_before_dot(source, offset)?;
    let ty = index.var_type_at(offset, &name)?;
    let fields = index.fields_for_type(&ty)?;
    if fields.is_empty() {
        return None;
    }
    let mut items = Vec::new();
    for field in fields {
        items.push(CompletionItem {
            label: field,
            kind: Some(CompletionItemKind::FIELD),
            ..CompletionItem::default()
        });
    }
    Some(items)
}

fn builtin_completion_items() -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for name in ["Option", "Result", "Object", "array", "int", "string", "bool", "float"] {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            ..CompletionItem::default()
        });
    }
    items
}

fn stdlib_completion_items() -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for name in ["panic", "is_valid_element", "create_root"] {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            ..CompletionItem::default()
        });
    }
    items
}

fn find_php_modules_root(start: &Path) -> Option<PathBuf> {
    if let Ok(root) = std::env::var("PHPX_MODULE_ROOT") {
        let root_path = PathBuf::from(root);
        let candidate = root_path.join("php_modules");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    let mut current = start.to_path_buf();
    if current.is_file() {
        current.pop();
    }
    loop {
        let candidate = current.join("php_modules");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn list_php_modules(root: &Path) -> Vec<String> {
    let mut modules = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return modules;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if name.starts_with('@') && path.is_dir() {
            if let Ok(children) = fs::read_dir(&path) {
                for child in children.flatten() {
                    let child_name = child.file_name().to_string_lossy().to_string();
                    if child.path().is_dir() {
                        modules.push(format!("{}/{}", name, child_name));
                    }
                }
            }
        } else if path.is_dir() {
            modules.push(name);
        }
    }
    modules.sort();
    modules
}

fn resolve_module_file(root: &Path, module_spec: &str, is_wasm: bool) -> Option<PathBuf> {
    let base = root.join(module_spec);
    if base.is_file() {
        return Some(base);
    }
    let phpx = base.with_extension("phpx");
    if phpx.is_file() {
        return Some(phpx);
    }
    let php = base.with_extension("php");
    if php.is_file() {
        return Some(php);
    }
    if base.is_dir() {
        if is_wasm {
            let stub = base.join("module.d.phpx");
            if stub.is_file() {
                return Some(stub);
            }
        }
        let index = base.join("index.phpx");
        if index.is_file() {
            return Some(index);
        }
        let index_php = base.join("index.php");
        if index_php.is_file() {
            return Some(index_php);
        }
        let module = base.join("module.phpx");
        if module.is_file() {
            return Some(module);
        }
    }
    None
}

fn collect_phpx_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if ext == "phpx" || ext == "php" {
                    files.push(path);
                }
            }
        }
    }
    files
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
    name.starts_with('.')
        || name == "node_modules"
        || name == "target"
        || name == "dist"
        || name == "build"
        || name == "vendor"
}

fn find_word_occurrences(source: &[u8], word: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let needle = word.as_bytes();
    if needle.is_empty() || needle.len() > source.len() {
        return spans;
    }
    let mut offset = 0usize;
    while offset + needle.len() <= source.len() {
        let Some(pos) = source[offset..]
            .windows(needle.len())
            .position(|window| window == needle)
        else {
            break;
        };
        let start = offset + pos;
        let end = start + needle.len();
        let left_ok = start == 0 || !is_ident_char(source[start - 1]);
        let right_ok = end >= source.len() || !is_ident_char(source[end]);
        if left_ok && right_ok {
            spans.push(Span::new(start, end));
        }
        offset = end;
    }
    spans
}
