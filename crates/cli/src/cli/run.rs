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

fn try_run_deka_script(context: &Context) -> Option<i32> {
    let first = context.args.positionals.first()?;
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
    json.get("scripts")?
        .as_object()?
        .get(name)?
        .as_str()
        .map(|s| s.to_string())
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
