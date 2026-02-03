use bumpalo::Bump;
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};

use crate::validation::syntax::validate_syntax;
use crate::validation::ValidationResult;

pub fn compile_phpx<'a>(source: &str, _file_path: &str, arena: &'a Bump) -> ValidationResult<'a> {
    let lexer = Lexer::new(source.as_bytes());
    let mut parser = Parser::new_with_mode(lexer, arena, ParserMode::Phpx);
    let program = parser.parse_program();

    let errors = validate_syntax(source, &program);
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
