use std::collections::{HashMap, HashSet};

use super::{ErrorKind, Severity, ValidationError, ValidationWarning};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportKind {
    Phpx,
    Wasm,
}

#[derive(Debug, Clone)]
struct ImportSpec {
    #[allow(dead_code)]
    imported: String,
    local: String,
    from: String,
    #[allow(dead_code)]
    kind: ImportKind,
    line: usize,
    column: usize,
    line_text: String,
}

pub fn validate_imports(
    source: &str,
    file_path: &str,
) -> (Vec<ValidationError>, Vec<ValidationWarning>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let bounds = frontmatter_bounds(&lines);
    let scan_end = bounds.map(|(_, end)| end).unwrap_or(lines.len());

    let mut import_lines = HashSet::new();
    let mut import_specs = Vec::new();
    let mut saw_code = false;
    let mut in_block_comment = false;

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
            if saw_code {
                errors.push(import_error(
                    idx + 1,
                    find_column(line, "import"),
                    trimmed.len(),
                    format!(
                        "Import must appear before other code in {}.",
                        file_path
                    ),
                    "Move import statements to the top of the file.",
                ));
                continue;
            }

            match parse_import_line(trimmed, line, idx + 1, file_path) {
                Ok(mut specs) => {
                    import_lines.insert(idx);
                    import_specs.append(&mut specs);
                }
                Err(err) => {
                    errors.push(err);
                }
            }
            continue;
        }

        saw_code = true;
    }

    let mut seen_locals: HashMap<String, (usize, usize)> = HashMap::new();
    for spec in &import_specs {
        if let Some((line, column)) = seen_locals.get(&spec.local) {
            errors.push(import_error(
                spec.line,
                spec.column,
                spec.local.len().max(1),
                format!(
                    "Duplicate import '{}'. First declared at line {}, column {}.",
                    spec.local, line, column
                ),
                "Remove the duplicate import or rename the local binding.",
            ));
        } else {
            seen_locals.insert(spec.local.clone(), (spec.line, spec.column));
        }

        if spec.from.contains("../") || spec.from.contains("..\\") {
            errors.push(import_error(
                spec.line,
                find_column(&spec.line_text, &spec.from),
                spec.from.len().max(1),
                format!(
                    "Relative module paths using '..' are not supported ('{}').",
                    spec.from
                ),
                "Use a module name from php_modules/ instead of relative paths.",
            ));
        }
    }

    let searchable = strip_comments_and_strings(&strip_import_lines(source, &import_lines));
    for spec in &import_specs {
        if !is_ident_used(&searchable, &spec.local) {
            warnings.push(import_warning(
                spec.line,
                find_column(&spec.line_text, &spec.local),
                spec.local.len().max(1),
                format!("Unused import '{}'.", spec.local),
                "Remove the unused import or use it in your code.",
            ));
        }
    }

    (errors, warnings)
}

fn parse_import_line(
    line: &str,
    raw_line: &str,
    line_number: usize,
    file_path: &str,
) -> Result<Vec<ImportSpec>, ValidationError> {
    let rest = line
        .trim_start()
        .strip_prefix("import")
        .ok_or_else(|| {
            import_error(
                line_number,
                find_column(raw_line, "import"),
                line.trim().len(),
                format!("Invalid import syntax in {}.", file_path),
                "Use: import { name } from 'module'.",
            )
        })?
        .trim_start();

    let rest = rest
        .strip_prefix('{')
        .ok_or_else(|| {
            import_error(
                line_number,
                find_column(raw_line, "import"),
                line.trim().len(),
                format!("Invalid import syntax in {}.", file_path),
                "Expected '{' after import.",
            )
        })?;

    let close_idx = rest.find('}').ok_or_else(|| {
        import_error(
            line_number,
            find_column(raw_line, "import"),
            line.trim().len(),
            format!("Invalid import syntax in {}.", file_path),
            "Missing closing '}' in import specifiers.",
        )
    })?;

    let specifiers = &rest[..close_idx];
    let mut after = rest[close_idx + 1..].trim_start();
    after = after
        .strip_prefix("from")
        .ok_or_else(|| {
            import_error(
                line_number,
                find_column(raw_line, "import"),
                line.trim().len(),
                format!("Invalid import syntax in {}.", file_path),
                "Expected 'from' after import specifiers.",
            )
        })?
        .trim_start();

    let (from, mut after_from) = parse_quoted_string(after).ok_or_else(|| {
        import_error(
            line_number,
            find_column(raw_line, "from"),
            line.trim().len(),
            format!("Invalid import syntax in {}.", file_path),
            "Module specifier must be quoted: from 'module'.",
        )
    })?;

    after_from = after_from.trim_start();
    let mut kind = ImportKind::Phpx;
    if let Some(rest_after_as) = after_from.strip_prefix("as") {
        let kind_str = rest_after_as.trim_start();
        if !kind_str.starts_with("wasm") {
            return Err(import_error(
                line_number,
                find_column(raw_line, "as"),
                line.trim().len(),
                format!(
                    "Unknown import kind '{}' in {}.",
                    kind_str.split_whitespace().next().unwrap_or(""),
                    file_path
                ),
                "Use `as wasm` for wasm imports or omit `as`.",
            ));
        }
        kind = ImportKind::Wasm;
        after_from = kind_str.trim_start_matches("wasm").trim_start();
    }

    let after_from = after_from.trim_start_matches(';').trim();
    if !after_from.is_empty() {
        return Err(import_error(
            line_number,
            find_column(raw_line, after_from),
            after_from.len().max(1),
            format!("Invalid import syntax in {}.", file_path),
            "Unexpected tokens after import statement.",
        ));
    }

    let spec_list: Vec<&str> = specifiers
        .split(',')
        .map(|spec| spec.trim())
        .filter(|spec| !spec.is_empty())
        .collect();

    if spec_list.is_empty() {
        return Err(import_error(
            line_number,
            find_column(raw_line, "import"),
            line.trim().len(),
            format!("Empty import list in {}.", file_path),
            "Add at least one import specifier.",
        ));
    }

    let mut specs = Vec::new();
    for spec in spec_list {
        let parts: Vec<&str> = spec.split_whitespace().collect();
        let (imported, local) = match parts.as_slice() {
            [name] => (*name, *name),
            [name, as_kw, alias] if *as_kw == "as" => (*name, *alias),
            _ => {
                return Err(import_error(
                    line_number,
                    find_column(raw_line, spec),
                    spec.len().max(1),
                    format!("Invalid import specifier '{}' in {}.", spec, file_path),
                    "Use `name` or `name as alias` inside the import list.",
                ));
            }
        };

        if !is_ident(imported) || !is_ident(local) {
            return Err(import_error(
                line_number,
                find_column(raw_line, spec),
                spec.len().max(1),
                format!("Invalid import specifier '{}' in {}.", spec, file_path),
                "Import names must be valid identifiers.",
            ));
        }

        specs.push(ImportSpec {
            imported: imported.to_string(),
            local: local.to_string(),
            from: from.clone(),
            kind,
            line: line_number,
            column: find_column(raw_line, local),
            line_text: raw_line.to_string(),
        });
    }

    Ok(specs)
}

fn import_error(
    line: usize,
    column: usize,
    underline_length: usize,
    message: String,
    help_text: &str,
) -> ValidationError {
    ValidationError {
        kind: ErrorKind::ImportError,
        line,
        column,
        message,
        help_text: help_text.to_string(),
        underline_length: underline_length.max(1),
        severity: Severity::Error,
    }
}

fn import_warning(
    line: usize,
    column: usize,
    underline_length: usize,
    message: String,
    help_text: &str,
) -> ValidationWarning {
    ValidationWarning {
        kind: ErrorKind::ImportError,
        line,
        column,
        message,
        help_text: help_text.to_string(),
        underline_length: underline_length.max(1),
        severity: Severity::Warning,
    }
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn parse_quoted_string(input: &str) -> Option<(String, &str)> {
    let mut chars = input.char_indices();
    let (_, quote) = chars.next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    let mut end_idx = None;
    for (idx, ch) in chars {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            end_idx = Some(idx);
            break;
        }
        out.push(ch);
    }
    let end_idx = end_idx?;
    let rest = &input[end_idx + quote.len_utf8()..];
    Some((out, rest))
}

fn strip_php_tags_inline(line: &str) -> String {
    line.replace("<?phpx", "")
        .replace("<?php", "")
        .replace("<?", "")
        .replace("?>", "")
}

fn frontmatter_bounds(lines: &[&str]) -> Option<(usize, usize)> {
    if lines.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    if i >= lines.len() {
        return None;
    }
    let mut first = lines[i];
    if let Some(stripped) = first.strip_prefix('\u{feff}') {
        first = stripped;
    }
    if first.trim() != "---" {
        return None;
    }
    let start = i;
    i += 1;
    for idx in i..lines.len() {
        if lines[idx].trim() == "---" {
            return Some((start, idx));
        }
    }
    None
}

fn consume_comment_line(trimmed: &str, in_block: &mut bool) -> bool {
    if *in_block {
        if let Some(end_idx) = trimmed.find("*/") {
            let rest = trimmed[end_idx + 2..].trim();
            *in_block = false;
            return rest.is_empty();
        }
        return true;
    }

    if trimmed.starts_with("/*") {
        if trimmed.contains("*/") {
            return true;
        }
        *in_block = true;
        return true;
    }

    trimmed.starts_with("//") || trimmed.starts_with('#')
}

fn strip_import_lines(source: &str, import_lines: &HashSet<usize>) -> String {
    let mut out = String::new();
    for (idx, line) in source.lines().enumerate() {
        if import_lines.contains(&idx) {
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn strip_comments_and_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push('\n');
            }
            continue;
        }
        if in_block_comment {
            if ch == '*' {
                if let Some('/') = chars.peek().copied() {
                    chars.next();
                    in_block_comment = false;
                }
            }
            continue;
        }
        if in_single {
            if ch == '\\' {
                chars.next();
                continue;
            }
            if ch == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            if ch == '\\' {
                chars.next();
                continue;
            }
            if ch == '"' {
                in_double = false;
            }
            continue;
        }

        if ch == '/' {
            if let Some('/') = chars.peek().copied() {
                chars.next();
                in_line_comment = true;
                continue;
            }
            if let Some('*') = chars.peek().copied() {
                chars.next();
                in_block_comment = true;
                continue;
            }
        }
        if ch == '#' {
            in_line_comment = true;
            continue;
        }
        if ch == '\'' {
            in_single = true;
            continue;
        }
        if ch == '"' {
            in_double = true;
            continue;
        }

        out.push(ch);
    }
    out
}

fn is_ident_used(source: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let bytes = source.as_bytes();
    let needle = name.as_bytes();
    let mut idx = 0;
    while idx + needle.len() <= bytes.len() {
        if let Some(pos) = source[idx..].find(name) {
            let start = idx + pos;
            let end = start + needle.len();
            let before = if start == 0 { None } else { Some(bytes[start - 1]) };
            let after = if end >= bytes.len() { None } else { Some(bytes[end]) };
            let valid_before = before.map_or(true, |b| !is_ident_char(b));
            let valid_after = after.map_or(true, |b| !is_ident_char(b));
            if valid_before && valid_after {
                return true;
            }
            idx = end;
        } else {
            break;
        }
    }
    false
}

fn is_ident_char(byte: u8) -> bool {
    (byte as char).is_ascii_alphanumeric() || byte == b'_'
}

fn find_column(line: &str, needle: &str) -> usize {
    line.find(needle).map(|idx| idx + 1).unwrap_or(1)
}
