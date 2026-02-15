use core::{CommandSpec, Context, FlagSpec, Registry};
use serde_json::Value;
use std::process::Command;

const COMMAND: CommandSpec = CommandSpec {
    name: "run",
    category: "runtime",
    summary: "run the app",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
    registry.add_flag(FlagSpec {
        name: "--watch",
        aliases: &["-W"],
        description: "keep event loop alive (for long-running processes)",
    });
}

pub fn cmd(context: &Context) {
    if let Some(exit_code) = try_run_deka_script(context) {
        std::process::exit(exit_code);
    }
    runtime::run(context);
}

fn requested_script_name<'a>(context: &'a Context) -> Option<&'a str> {
    requested_script_name_from_args(&context.args.positionals, &context.args.commands)
}

fn requested_script_name_from_args<'a>(positionals: &'a [String], commands: &'a [String]) -> Option<&'a str> {
    if let Some(first) = positionals.first() {
        return Some(first.as_str());
    }
    if commands.len() > 1 {
        return Some(commands[1].as_str());
    }
    None
}

fn try_run_deka_script(context: &Context) -> Option<i32> {
    let first = requested_script_name(context)?;
    if first.contains('/') || first.contains('\\') || first.starts_with('.') {
        return None;
    }
    let script = resolve_deka_script(first)?;
    let extra_args = if context.args.positionals.len() > 1 {
        &context.args.positionals[1..]
    } else {
        &[]
    };
    let command = compose_script_command(&script, extra_args);
    run_shell_command(&command).or(Some(1))
}

fn resolve_deka_script(name: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let path = cwd.join("deka.json");
    let content = std::fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;
    resolve_deka_script_from_json(name, &json)
}

fn resolve_deka_script_from_json(name: &str, json: &Value) -> Option<String> {
    if let Some(script) = json
        .get("scripts")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get(name))
        .and_then(|v| v.as_str())
    {
        return Some(script.to_string());
    }

    default_project_script(name, json)
}

fn default_project_script(name: &str, json: &Value) -> Option<String> {
    let project_type = json
        .get("type")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_ascii_lowercase());

    if name == "build" {
        if project_type.as_deref() == Some("serve") {
            return Some("deka build".to_string());
        }
        return None;
    }

    if name != "dev" {
        return None;
    }

    if project_type.as_deref() != Some("serve") {
        return None;
    }

    let has_entry = json
        .get("serve")
        .and_then(|v| v.get("entry"))
        .and_then(|v| v.as_str())
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    if !has_entry {
        return None;
    }

    Some("deka serve --dev".to_string())
}

fn compose_script_command(script: &str, extra_args: &[String]) -> String {
    if extra_args.is_empty() {
        return script.to_string();
    }
    let mut cmd = String::with_capacity(script.len() + 1 + extra_args.iter().map(|s| s.len() + 1).sum::<usize>());
    cmd.push_str(script);
    for arg in extra_args {
        cmd.push(' ');
        if arg.contains(char::is_whitespace) || arg.contains('"') || arg.contains('\'') {
            let escaped = arg.replace('"', "\\\"");
            cmd.push('"');
            cmd.push_str(&escaped);
            cmd.push('"');
        } else {
            cmd.push_str(arg);
        }
    }
    cmd
}

fn run_shell_command(command: &str) -> Option<i32> {
    let status = if cfg!(windows) {
        Command::new("cmd").args(["/C", command]).status().ok()?
    } else {
        Command::new("/bin/zsh")
            .args(["-lc", command])
            .status()
            .ok()?
    };
    Some(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_explicit_script_first() {
        let json: Value = serde_json::json!({
            "type": "serve",
            "serve": { "entry": "app/main.phpx" },
            "scripts": { "dev": "deka serve --dev --watch" }
        });
        let script = resolve_deka_script_from_json("dev", &json).expect("script");
        assert_eq!(script, "deka serve --dev --watch");
    }

    #[test]
    fn resolves_default_dev_for_serve_projects() {
        let json: Value = serde_json::json!({
            "type": "serve",
            "serve": { "entry": "app/main.phpx" }
        });
        let script = resolve_deka_script_from_json("dev", &json).expect("script");
        assert_eq!(script, "deka serve --dev");
    }

    #[test]
    fn does_not_resolve_default_dev_for_lib_projects() {
        let json: Value = serde_json::json!({
            "type": "lib",
            "serve": { "entry": "app/main.phpx" }
        });
        assert!(resolve_deka_script_from_json("dev", &json).is_none());
    }

    #[test]
    fn resolves_default_build_for_serve_projects() {
        let json: Value = serde_json::json!({
            "type": "serve",
            "serve": { "entry": "app/main.phpx" }
        });
        let script = resolve_deka_script_from_json("build", &json).expect("script");
        assert_eq!(script, "deka build");
    }

    #[test]
    fn does_not_resolve_default_build_for_lib_projects() {
        let json: Value = serde_json::json!({
            "type": "lib",
            "serve": { "entry": "app/main.phpx" }
        });
        assert!(resolve_deka_script_from_json("build", &json).is_none());
    }

    #[test]
    fn requested_script_prefers_positionals() {
        let positionals = vec!["dev".to_string()];
        let commands = vec!["run".to_string(), "build".to_string()];
        assert_eq!(requested_script_name_from_args(&positionals, &commands), Some("dev"));
    }

    #[test]
    fn requested_script_falls_back_to_second_command_token() {
        let positionals: Vec<String> = vec![];
        let commands = vec!["run".to_string(), "build".to_string()];
        assert_eq!(requested_script_name_from_args(&positionals, &commands), Some("build"));
    }
}
