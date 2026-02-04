use std::collections::HashSet;

use php_rs::parser::ast::visitor::{walk_expr, Visitor};
use php_rs::parser::ast::{Expr, ExprId, JsxAttribute, JsxChild, Name, Program, Stmt};
use php_rs::parser::span::Span;

use super::{ErrorKind, Severity, ValidationError};
use crate::validation::imports::{
    consume_comment_line, find_column, frontmatter_bounds, parse_import_line, strip_php_tags_inline,
};

pub fn validate_jsx_syntax(program: &Program, source: &str) -> Vec<ValidationError> {
    let mut validator = JsxSyntaxValidator {
        source,
        errors: Vec::new(),
    };
    validator.visit_program(program);
    validator.errors
}

pub fn validate_jsx_expressions(program: &Program, source: &str) -> Vec<ValidationError> {
    let mut validator = JsxExpressionValidator {
        source,
        errors: Vec::new(),
    };
    validator.visit_program(program);
    validator.errors
}

pub fn validate_components(program: &Program, source: &str) -> Vec<ValidationError> {
    let mut known_components = collect_function_names(program, source);
    known_components.extend(collect_imported_names(source));

    let mut validator = JsxComponentValidator {
        source,
        known_components,
        errors: Vec::new(),
    };
    validator.visit_program(program);
    validator.errors
}

pub fn validate_frontmatter(source: &str, file_path: &str) -> Vec<ValidationError> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors = Vec::new();
    let mut saw_frontmatter = false;

    let mut first_non_empty = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        first_non_empty = Some(idx);
        break;
    }

    if let Some(idx) = first_non_empty {
        let mut line = lines[idx].trim();
        if let Some(stripped) = line.strip_prefix('\u{feff}') {
            line = stripped;
        }
        if line == "---" {
            saw_frontmatter = true;
        } else if lines.iter().skip(idx + 1).any(|line| line.trim() == "---") {
            errors.push(frontmatter_error(
                idx + 1,
                find_column(lines[idx], line),
                line.len().max(1),
                "Frontmatter must start at the beginning of the file.".to_string(),
                "Move the '---' delimiter to the top of the file.",
            ));
        }
    }

    if !saw_frontmatter {
        return errors;
    }

    let bounds = frontmatter_bounds(&lines);
    if bounds.is_none() {
        errors.push(frontmatter_error(
            1,
            1,
            3,
            "Frontmatter delimiter '---' is missing a closing delimiter.".to_string(),
            "Add a closing '---' to end the frontmatter section.",
        ));
        return errors;
    }
    let (_start, end) = bounds.unwrap();

    let mut in_block_comment = false;
    for (idx, line) in lines.iter().enumerate().skip(end + 1) {
        let clean = strip_php_tags_inline(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() {
            continue;
        }
        if consume_comment_line(trimmed, &mut in_block_comment) {
            continue;
        }
        if trimmed.starts_with("import ") {
            errors.push(frontmatter_error(
                idx + 1,
                find_column(line, "import"),
                trimmed.len().max(1),
                "Imports must appear in the frontmatter section.".to_string(),
                "Move the import statement above the closing '---'.",
            ));
        }
        if file_path.contains("php_modules/") && trimmed.starts_with("export ") {
            errors.push(frontmatter_error(
                idx + 1,
                find_column(line, "export"),
                trimmed.len().max(1),
                "Template files cannot declare exports.".to_string(),
                "Remove the export statement from the template section.",
            ));
        }
    }

    errors
}

struct JsxSyntaxValidator<'a> {
    source: &'a str,
    errors: Vec<ValidationError>,
}

impl<'ast> Visitor<'ast> for JsxSyntaxValidator<'_> {
    fn visit_expr(&mut self, expr: ExprId<'ast>) {
        match expr {
            Expr::JsxElement { name, attributes, .. } => {
                self.check_tag_name(name);
                for attr in *attributes {
                    self.check_attribute(attr);
                }
            }
            Expr::JsxFragment { .. } => {}
            _ => {}
        }
        walk_expr(self, expr);
    }
}

impl JsxSyntaxValidator<'_> {
    fn check_tag_name(&mut self, name: &Name) {
        let Some(raw) = name_to_string(name, self.source) else {
            return;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            self.errors.push(jsx_error(
                name.span,
                self.source,
                "JSX tag name cannot be empty.".to_string(),
                "Provide a valid tag or component name.",
            ));
        }
    }

    fn check_attribute(&mut self, attr: &JsxAttribute) {
        let Some(name) = token_text(attr.name, self.source) else {
            return;
        };
        if !is_valid_attr_name(&name) {
            self.errors.push(jsx_error(
                attr.span,
                self.source,
                format!("Invalid JSX attribute name '{}'.", name),
                "Use a valid attribute identifier.",
            ));
        }
    }
}

struct JsxExpressionValidator<'a> {
    source: &'a str,
    errors: Vec<ValidationError>,
}

impl<'ast> Visitor<'ast> for JsxExpressionValidator<'_> {
    fn visit_expr(&mut self, expr: ExprId<'ast>) {
        match expr {
            Expr::JsxElement {
                attributes,
                children,
                ..
            } => {
                for attr in *attributes {
                    if let Some(value) = attr.value {
                        if !is_jsx_expr(value) {
                            self.validate_expr(value);
                        }
                    }
                }
                for child in *children {
                    if let JsxChild::Expr(child_expr) = child {
                        if !is_jsx_expr(child_expr) {
                            self.validate_expr(child_expr);
                        }
                    }
                }
            }
            Expr::JsxFragment { children, .. } => {
                for child in *children {
                    if let JsxChild::Expr(child_expr) = child {
                        if !is_jsx_expr(child_expr) {
                            self.validate_expr(child_expr);
                        }
                    }
                }
            }
            _ => {}
        }
        walk_expr(self, expr);
    }
}

impl JsxExpressionValidator<'_> {
    fn validate_expr(&mut self, expr: ExprId<'_>) {
        let mut validator = JsxExprValidator {
            source: self.source,
            errors: Vec::new(),
        };
        validator.visit_expr(expr);
        self.errors.extend(validator.errors);
    }
}

struct JsxExprValidator<'a> {
    source: &'a str,
    errors: Vec<ValidationError>,
}

impl<'ast> Visitor<'ast> for JsxExprValidator<'_> {
    fn visit_expr(&mut self, expr: ExprId<'ast>) {
        match expr {
            Expr::Assign { span, .. }
            | Expr::AssignRef { span, .. }
            | Expr::AssignOp { span, .. } => {
                self.errors.push(jsx_error(
                    *span,
                    self.source,
                    "Statements are not allowed inside JSX expressions.".to_string(),
                    "Use a value expression instead of an assignment.",
                ));
            }
            Expr::Yield { span, .. } => {
                self.errors.push(jsx_error(
                    *span,
                    self.source,
                    "Statements are not allowed inside JSX expressions.".to_string(),
                    "Remove the yield statement from JSX.",
                ));
            }
            Expr::Error { span } => {
                self.errors.push(jsx_error(
                    *span,
                    self.source,
                    "Invalid JSX expression.".to_string(),
                    "Fix the expression inside the JSX braces.",
                ));
            }
            _ => {}
        }
        walk_expr(self, expr);
    }
}

struct JsxComponentValidator<'a> {
    source: &'a str,
    known_components: HashSet<String>,
    errors: Vec<ValidationError>,
}

impl<'ast> Visitor<'ast> for JsxComponentValidator<'_> {
    fn visit_expr(&mut self, expr: ExprId<'ast>) {
        if let Expr::JsxElement { name, attributes, .. } = expr {
            self.validate_element(name, attributes);
        }
        walk_expr(self, expr);
    }
}

impl JsxComponentValidator<'_> {
    fn validate_element(&mut self, name: &Name, attributes: &[JsxAttribute]) {
        let Some(raw) = name_to_string(name, self.source) else {
            return;
        };
        let trimmed = raw.trim_start_matches('\\');
        let last = trimmed.rsplit('\\').next().unwrap_or(trimmed);
        if last.is_empty() {
            return;
        }

        let mut chars = last.chars();
        let is_component = chars.next().map(|ch| ch.is_ascii_uppercase()).unwrap_or(false);
        let has_uppercase = last.chars().any(|ch| ch.is_ascii_uppercase());

        if !is_component && has_uppercase {
            self.errors.push(jsx_error(
                name.span,
                self.source,
                format!(
                    "JSX component '{}' must be capitalized (use <{} />).",
                    last,
                    capitalize_jsx_name(last)
                ),
                "Use a capitalized component name for components.",
            ));
            return;
        }

        if is_component && !self.known_components.contains(last) {
            self.errors.push(jsx_error(
                name.span,
                self.source,
                format!(
                    "Unknown component '{}'; import it or define function {}().",
                    last, last
                ),
                "Import the component or define a matching function.",
            ));
            return;
        }

        if is_component {
            self.validate_props(last, attributes, name.span);
        }
    }

    fn validate_props(&mut self, component: &str, attributes: &[JsxAttribute], span: Span) {
        let mut attrs = HashSet::new();
        for attr in attributes {
            if let Some(name) = token_text(attr.name, self.source) {
                attrs.insert(name);
            }
        }

        match component {
            "Link" => {
                if !attrs.contains("to") {
                    self.errors.push(jsx_error(
                        span,
                        self.source,
                        "Link requires prop 'to'.".to_string(),
                        "Add a `to` attribute to Link.",
                    ));
                }
            }
            "ContextProvider" => {
                if !attrs.contains("ctx") {
                    self.errors.push(jsx_error(
                        span,
                        self.source,
                        "ContextProvider requires prop 'ctx'.".to_string(),
                        "Add a `ctx` attribute to ContextProvider.",
                    ));
                }
                if !attrs.contains("value") {
                    self.errors.push(jsx_error(
                        span,
                        self.source,
                        "ContextProvider requires prop 'value'.".to_string(),
                        "Add a `value` attribute to ContextProvider.",
                    ));
                }
            }
            _ => {}
        }
    }
}

fn collect_function_names(program: &Program, source: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for stmt in program.statements {
        if let Stmt::Function { name, .. } = stmt {
            if let Some(text) = token_text(name, source) {
                names.insert(text);
            }
        }
    }
    names
}

fn collect_imported_names(source: &str) -> HashSet<String> {
    let lines: Vec<&str> = source.lines().collect();
    let bounds = frontmatter_bounds(&lines);
    let scan_end = bounds.map(|(_, end)| end).unwrap_or(lines.len());
    let mut in_block_comment = false;
    let mut imported = HashSet::new();
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
            if let Ok(specs) = parse_import_line(trimmed, line, idx + 1, "<source>") {
                for spec in specs {
                    imported.insert(spec.local);
                }
            }
        }
    }
    imported
}

fn jsx_error(span: Span, source: &str, message: String, help_text: &str) -> ValidationError {
    let (line, column, underline_length) = span_location(span, source);
    ValidationError {
        kind: ErrorKind::JsxError,
        line,
        column,
        message,
        help_text: help_text.to_string(),
        underline_length,
        severity: Severity::Error,
    }
}

fn frontmatter_error(
    line: usize,
    column: usize,
    underline_length: usize,
    message: String,
    help_text: &str,
) -> ValidationError {
    ValidationError {
        kind: ErrorKind::JsxError,
        line,
        column,
        message,
        help_text: help_text.to_string(),
        underline_length: underline_length.max(1),
        severity: Severity::Error,
    }
}

fn span_location(span: Span, source: &str) -> (usize, usize, usize) {
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

fn name_to_string(name: &Name, source: &str) -> Option<String> {
    std::str::from_utf8(name.span.as_str(source.as_bytes()))
        .ok()
        .map(|text| text.to_string())
}

fn token_text(token: &php_rs::parser::lexer::token::Token, source: &str) -> Option<String> {
    std::str::from_utf8(token.text(source.as_bytes()))
        .ok()
        .map(|text| text.to_string())
}

fn is_valid_attr_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric() || ch == '-')
}

fn capitalize_jsx_name(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.push(first.to_ascii_uppercase());
    out.push_str(chars.as_str());
    out
}

fn is_jsx_expr(expr: ExprId<'_>) -> bool {
    matches!(expr, Expr::JsxElement { .. } | Expr::JsxFragment { .. })
}
