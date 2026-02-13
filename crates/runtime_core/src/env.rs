use std::collections::HashMap;

pub fn flag_or_env_truthy(
    flags: &HashMap<String, bool>,
    long_flag: &str,
    short_flag: Option<&str>,
    env_var: &str,
) -> bool {
    if flags.contains_key(long_flag) {
        return true;
    }
    if let Some(short) = short_flag {
        if flags.contains_key(short) {
            return true;
        }
    }
    env_truthy(env_var)
}

pub fn env_truthy(var: &str) -> bool {
    std::env::var(var)
        .map(|value| is_truthy(&value))
        .unwrap_or(false)
}

pub fn is_truthy(value: &str) -> bool {
    matches!(value, "1" | "true" | "yes" | "on")
}
