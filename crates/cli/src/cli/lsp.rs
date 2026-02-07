use core::{CommandSpec, Context, Registry};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const COMMAND: CommandSpec = CommandSpec {
    name: "lsp",
    category: "tooling",
    summary: "run the PHPX language server",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

fn resolve_lsp_binary() -> Option<PathBuf> {
    if let Ok(bin) = std::env::var("PHPX_LSP_BIN") {
        if !bin.is_empty() {
            return Some(PathBuf::from(bin));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(lsp_binary_name());
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    Some(PathBuf::from(lsp_binary_name()))
}

fn lsp_binary_name() -> &'static str {
    if cfg!(windows) {
        "phpx_lsp.exe"
    } else {
        "phpx_lsp"
    }
}

pub fn cmd(context: &Context) {
    let bin = match resolve_lsp_binary() {
        Some(bin) => bin,
        None => {
            stdio::error("cli", "phpx_lsp binary not found");
            return;
        }
    };

    let mut cmd = Command::new(bin);
    cmd.args(&context.args.positionals)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match cmd.status() {
        Ok(status) => {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
        }
        Err(err) => {
            stdio::error("cli", &format!("failed to start phpx_lsp: {}", err));
            std::process::exit(1);
        }
    }
}
