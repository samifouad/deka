use std::path::Path;

pub fn handler_input_with<Get>(
    positionals: &[String],
    env_get: &Get,
) -> (String, Vec<String>)
where
    Get: Fn(&str) -> Option<String>,
{
    let handler = positionals
        .first()
        .cloned()
        .or_else(|| env_get("HANDLER_PATH"))
        .unwrap_or_else(|| ".".to_string());
    let extra_args = if positionals.len() > 1 {
        positionals[1..].to_vec()
    } else {
        Vec::new()
    };
    (handler, extra_args)
}

pub fn normalize_handler_path(path: &str) -> String {
    let path = Path::new(path);
    if path.is_absolute() {
        return path.to_string_lossy().to_string();
    }
    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return path.to_string_lossy().to_string(),
    };
    let joined = cwd.join(path);
    match joined.canonicalize() {
        Ok(canon) => canon.to_string_lossy().to_string(),
        Err(_) => joined.to_string_lossy().to_string(),
    }
}

pub fn is_php_entry(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    lowered.ends_with(".php") || lowered.ends_with(".phpx")
}

pub fn is_html_entry(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".html")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn handler_input_prefers_first_positional() {
        let env = HashMap::<String, String>::from([("HANDLER_PATH".into(), "env.phpx".into())]);
        let env_get = |k: &str| env.get(k).cloned();
        let (handler, extra) = handler_input_with(&["main.phpx".into(), "a".into()], &env_get);
        assert_eq!(handler, "main.phpx");
        assert_eq!(extra, vec!["a".to_string()]);
    }

    #[test]
    fn handler_input_uses_env_then_default() {
        let env = HashMap::<String, String>::from([("HANDLER_PATH".into(), "env.phpx".into())]);
        let env_get = |k: &str| env.get(k).cloned();
        let (handler, extra) = handler_input_with(&[], &env_get);
        assert_eq!(handler, "env.phpx");
        assert!(extra.is_empty());

        let none_get = |_k: &str| None;
        let (handler2, extra2) = handler_input_with(&[], &none_get);
        assert_eq!(handler2, ".");
        assert!(extra2.is_empty());
    }

    #[test]
    fn php_and_html_detection() {
        assert!(is_php_entry("index.php"));
        assert!(is_php_entry("index.PHPX"));
        assert!(!is_php_entry("index.html"));
        assert!(is_html_entry("index.html"));
        assert!(is_html_entry("index.HTML"));
    }
}
