pub fn validate_phpx_handler_with<E, ReadSource, Validate, FormatError>(
    handler_path: &str,
    read_source: &ReadSource,
    validate: &Validate,
    format_error: &FormatError,
) -> Result<(), String>
where
    ReadSource: Fn(&str) -> Result<String, String>,
    Validate: Fn(&str, &str) -> Vec<E>,
    FormatError: Fn(&str, &str, &E) -> String,
{
    if !handler_path.to_ascii_lowercase().ends_with(".phpx") {
        return Ok(());
    }

    let source = read_source(handler_path)?;
    let errors = validate(&source, handler_path);
    if errors.is_empty() {
        return Ok(());
    }

    let mut out = String::new();
    for error in errors.iter().take(3) {
        out.push_str(&format_error(&source, handler_path, error));
    }
    if errors.len() > 3 {
        out.push_str(&format!(
            "\n... plus {} additional module validation error(s)\n",
            errors.len() - 3
        ));
    }

    Err(format!(
        "PHPX module graph validation failed for {}:\n{}",
        handler_path, out
    ))
}

#[cfg(test)]
mod tests {
    use super::validate_phpx_handler_with;

    #[test]
    fn skips_non_phpx_paths() {
        let result = validate_phpx_handler_with(
            "index.php",
            &|_| Ok(String::new()),
            &|_, _| vec![1],
            &|_, _, _| "err".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn returns_error_with_limited_output() {
        let result = validate_phpx_handler_with(
            "index.phpx",
            &|_| Ok("source".to_string()),
            &|_, _| vec![1, 2, 3, 4],
            &|_, _, e| format!("error:{}\n", e),
        );
        let err = result.expect_err("expected validation error");
        assert!(err.contains("PHPX module graph validation failed"));
        assert!(err.contains("error:1"));
        assert!(err.contains("error:3"));
        assert!(err.contains("plus 1 additional"));
    }
}
