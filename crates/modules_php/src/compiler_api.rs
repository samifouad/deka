use bumpalo::Bump;
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};

use crate::validation::{parse_errors_to_validation_errors, ValidationResult};

pub fn compile_phpx<'a>(source: &str, _file_path: &str, arena: &'a Bump) -> ValidationResult<'a> {
    let lexer = Lexer::new(source.as_bytes());
    let mut parser = Parser::new_with_mode(lexer, arena, ParserMode::Phpx);
    let program = parser.parse_program();

    let errors = parse_errors_to_validation_errors(source, program.errors);
    if !errors.is_empty() {
        return ValidationResult {
            errors,
            warnings: Vec::new(),
            ast: None,
        };
    }

    ValidationResult {
        errors,
        warnings: Vec::new(),
        ast: Some(program),
    }
}
