use core::{CommandSpec, Context, Registry, SubcommandSpec};
use std::io::{self, Write};
use stdio;

use crate::cli::auth_store::{self, AuthProfile};

const AUTH_COMMAND: CommandSpec = CommandSpec {
    name: "auth",
    category: "auth",
    summary: "authenticate with linkhash",
    aliases: &[],
    subcommands: &[LOGIN_SUBCOMMAND, LOGOUT_SUBCOMMAND, WHOAMI_SUBCOMMAND],
    handler: cmd_auth,
};

const LOGIN_SUBCOMMAND: SubcommandSpec = SubcommandSpec {
    name: "login",
    summary: "store username/token for deka commands",
    aliases: &[],
    handler: cmd_login,
};

const LOGOUT_SUBCOMMAND: SubcommandSpec = SubcommandSpec {
    name: "logout",
    summary: "clear saved credentials",
    aliases: &[],
    handler: cmd_logout,
};

const WHOAMI_SUBCOMMAND: SubcommandSpec = SubcommandSpec {
    name: "whoami",
    summary: "show current authenticated user",
    aliases: &[],
    handler: cmd_whoami,
};

const LOGIN_ALIAS_COMMAND: CommandSpec = CommandSpec {
    name: "login",
    category: "auth",
    summary: "alias for `deka auth login`",
    aliases: &[],
    subcommands: &[],
    handler: cmd_login,
};

const LOGOUT_ALIAS_COMMAND: CommandSpec = CommandSpec {
    name: "logout",
    category: "auth",
    summary: "alias for `deka auth logout`",
    aliases: &[],
    subcommands: &[],
    handler: cmd_logout,
};

const WHOAMI_ALIAS_COMMAND: CommandSpec = CommandSpec {
    name: "whoami",
    category: "auth",
    summary: "alias for `deka auth whoami`",
    aliases: &[],
    subcommands: &[],
    handler: cmd_whoami,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(AUTH_COMMAND);
    registry.add_command(LOGIN_ALIAS_COMMAND);
    registry.add_command(LOGOUT_ALIAS_COMMAND);
    registry.add_command(WHOAMI_ALIAS_COMMAND);
}

fn cmd_auth(_context: &Context) {
    stdio::log("auth", "available subcommands: login, logout, whoami");
}

fn cmd_login(context: &Context) {
    let username_raw = context
        .args
        .params
        .get("--username")
        .cloned()
        .or_else(|| prompt_required("Username (@username): "));
    let token = context
        .args
        .params
        .get("--token")
        .cloned()
        .or_else(|| std::env::var("LINKHASH_TOKEN").ok())
        .or_else(|| prompt_required("Token: "));
    let registry_url = context
        .args
        .params
        .get("--registry-url")
        .cloned()
        .or_else(|| std::env::var("LINKHASH_REGISTRY_URL").ok())
        .or_else(|| {
            prompt_optional(
                "Registry URL [http://localhost:8508]: ",
                Some("http://localhost:8508"),
            )
        })
        .unwrap_or_else(|| "http://localhost:8508".to_string());

    let Some(token) = token else {
        stdio::error("auth", "missing token");
        return;
    };

    let Some(username_raw) = username_raw else {
        stdio::error("auth", "missing username (expected @username)");
        return;
    };

    let username = normalize_username(&username_raw);
    if !is_valid_username(&username) {
        stdio::error("auth", "invalid --username (expected @username)");
        return;
    }

    let profile = AuthProfile {
        username: username.clone(),
        token,
        registry_url,
    };

    if let Err(err) = auth_store::save(&profile) {
        stdio::error("auth", &format!("failed to persist auth profile: {}", err));
        return;
    }

    stdio::log("auth", &format!("logged in as {}", username));
}

fn cmd_logout(_context: &Context) {
    match auth_store::clear() {
        Ok(true) => stdio::log("auth", "logged out"),
        Ok(false) => stdio::log("auth", "no active auth profile"),
        Err(err) => stdio::error("auth", &format!("failed to clear auth profile: {}", err)),
    }
}

fn cmd_whoami(_context: &Context) {
    match auth_store::load() {
        Ok(Some(profile)) => {
            stdio::log(
                "auth",
                &format!("{} ({})", profile.username, profile.registry_url),
            );
        }
        Ok(None) => stdio::error("auth", "not logged in (run `deka login`)"),
        Err(err) => stdio::error("auth", &format!("failed to read auth profile: {}", err)),
    }
}

pub fn normalize_username(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('@') {
        trimmed.to_string()
    } else {
        format!("@{}", trimmed)
    }
}

fn is_valid_username(username: &str) -> bool {
    if !username.starts_with('@') || username.len() < 2 {
        return false;
    }

    username[1..]
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn prompt_required(prompt: &str) -> Option<String> {
    let value = prompt_optional(prompt, None)?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn prompt_optional(prompt: &str, default_value: Option<&str>) -> Option<String> {
    print!("{}", prompt);
    if io::stdout().flush().is_err() {
        return None;
    }

    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_err() {
        return None;
    }

    let trimmed = buf.trim().to_string();
    if trimmed.is_empty() {
        return default_value.map(|s| s.to_string());
    }
    Some(trimmed)
}
