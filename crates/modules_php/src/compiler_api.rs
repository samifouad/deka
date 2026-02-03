use bumpalo::Bump;
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};

use crate::validation::imports::validate_imports;
use crate::validation::syntax::validate_syntax;
use crate::validation::ValidationResult;

pub fn compile_phpx<'a>(source: &str, file_path: &str, arena: &'a Bump) -> ValidationResult<'a> {
    let lexer = Lexer::new(source.as_bytes());
    let mut parser = Parser::new_with_mode(lexer, arena, ParserMode::Phpx);
    let program = parser.parse_program();

    let mut errors = validate_syntax(source, &program);
    let mut warnings = Vec::new();
    if !errors.is_empty() {
        return ValidationResult {
            errors,
            warnings,
            ast: None,
        };
    }

    let (import_errors, import_warnings) = validate_imports(source, file_path);
    errors.extend(import_errors);
    warnings.extend(import_warnings);
    if !errors.is_empty() {
        return ValidationResult {
            errors,
            warnings,
            ast: None,
        };
    }

    ValidationResult {
        errors,
        warnings,
        ast: Some(program),
    }
}
