use php_rs::parser::ast::Program;

use super::{parse_errors_to_validation_errors, ValidationError};

const DEFAULT_HELP_TEXT: &str = "Check syntax near the highlighted code.";

pub fn validate_syntax(source: &str, program: &Program) -> Vec<ValidationError> {
    let mut errors = parse_errors_to_validation_errors(source, program.errors);

    for error in &mut errors {
        improve_help_text(error);
    }

    errors
}

fn improve_help_text(error: &mut ValidationError) {
    if error.help_text != DEFAULT_HELP_TEXT {
        return;
    }

    let message = error.message.as_str();
    if message.contains("Expected ';'") {
        error.help_text = "Add a semicolon at the end of the statement.".to_string();
        return;
    }

    if message.contains("Expected '}'") {
        error.help_text = "Add a closing '}' for the block.".to_string();
        return;
    }

    if message.contains("Expected ')'") {
        error.help_text = "Add a closing ')' for the expression.".to_string();
        return;
    }

    if message.contains("Expected ']'") {
        error.help_text = "Add a closing ']' for the array or index access.".to_string();
        return;
    }

    if message.contains("Unexpected end of file") || message.contains("end of file") {
        error.help_text = "Check for missing closing braces or parentheses.".to_string();
        return;
    }

    if message.contains("Unexpected token") {
        error.help_text = "Remove the unexpected token or fix the surrounding syntax.".to_string();
        return;
    }

    if message.contains("Invalid token") {
        error.help_text = "Remove the invalid token or replace it with valid PHPX syntax.".to_string();
    }
}
