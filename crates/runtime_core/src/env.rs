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

pub fn set_if_absent(key: &str, value: &str) {
    if std::env::var(key).is_ok() {
        return;
    }
    unsafe {
        std::env::set_var(key, value);
    }
}

pub fn set_dev_flag(enabled: bool) {
    if enabled {
        set_if_absent("DEKA_DEV", "1");
    }
}

pub fn set_handler_path(handler_path: &str) {
    set_if_absent("HANDLER_PATH", handler_path);
}

pub fn set_default_log_level() {
    set_if_absent("LOG_LEVEL", "error");
}

pub fn set_runtime_args(extra_args: &[String]) {
    if !extra_args.is_empty() {
        if let Ok(encoded) = serde_json::to_string(extra_args) {
            unsafe {
                std::env::set_var("DEKA_ARGS", encoded);
            }
        }
    }

    if let Some(bin) = std::env::args().next() {
        unsafe {
            std::env::set_var("DEKA_BIN", bin);
        }
    }
}
