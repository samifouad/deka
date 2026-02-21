use anyhow::Result;
use core::{CommandSpec, Context, FlagSpec, ParamSpec, Registry};
use pm::{InstallPayload, run_install};
use std::path::PathBuf;
use stdio;

const INSTALL_COMMAND: CommandSpec = CommandSpec {
    name: "install",
    category: "package",
    summary: "install dependencies via the package manager",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

const ADD_COMMAND: CommandSpec = CommandSpec {
    name: "add",
    category: "package",
    summary: "install php package(s) (shorthand)",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

const I_COMMAND: CommandSpec = CommandSpec {
    name: "i",
    category: "package",
    summary: "install php package(s) (short alias)",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(INSTALL_COMMAND);
    registry.add_command(ADD_COMMAND);
    registry.add_command(I_COMMAND);
    registry.add_flag(FlagSpec {
        name: "--quiet",
        aliases: &["-q"],
        description: "suppress informational output",
    });
    registry.add_flag(FlagSpec {
        name: "--yes",
        aliases: &["-y"],
        description: "assume yes when prompted",
    });
    registry.add_flag(FlagSpec {
        name: "--prompt",
        aliases: &["-p"],
        description: "prompt before installing",
    });
    registry.add_flag(FlagSpec {
        name: "--rehash",
        aliases: &[],
        description: "rehash php package integrity and update deka.lock",
    });
    registry.add_param(ParamSpec {
        name: "--payload",
        description: "path to a JSON payload describing the install",
    });
    registry.add_param(ParamSpec {
        name: "--ecosystem",
        description: "ecosystem hint (php)",
    });
    registry.add_param(ParamSpec {
        name: "--spec",
        description: "package spec or comma-separated list of specs",
    });
    registry.add_param(ParamSpec {
        name: "--concurrency",
        description: "number of concurrent downloads (ignored for now)",
    });
}

pub fn cmd(context: &Context) {
    match build_payload(context) {
        Ok(payload) => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            if let Err(err) = runtime.block_on(run_install(payload)) {
                let message = err.to_string();
                stdio::error("install", &message);
            }
        }
        Err(err) => {
            let message = err.to_string();
            stdio::error("install", &message);
        }
    }
}

fn build_payload(context: &Context) -> Result<InstallPayload> {
    let command_name = context
        .args
        .commands
        .first()
        .map(String::as_str)
        .unwrap_or("install");

    let mut payload = if let Some(payload_path) = context.args.params.get("--payload") {
        let mut payload = InstallPayload::from_file(&PathBuf::from(payload_path))?;
        if let Some(path) = context.args.params.get("--ecosystem") {
            payload.ecosystem = Some(path.clone());
        }
        payload
    } else {
        let mut specs = context
            .args
            .params
            .get("--spec")
            .map(|value| parse_spec_list(value))
            .unwrap_or_default();
        if specs.is_empty() && !context.args.positionals.is_empty() {
            specs = context.args.positionals.clone();
        }
        let ecosystem = context.args.params.get("--ecosystem").cloned().or_else(|| {
            if command_name == "install" {
                None
            } else {
                Some("php".to_string())
            }
        });
        InstallPayload {
            specs,
            ecosystem,
            yes: false,
            prompt: false,
            quiet: false,
            rehash: false,
        }
    };

    if payload.ecosystem.as_deref() == Some("php") {
        let mut resolved_specs = Vec::new();
        for spec in &payload.specs {
            resolved_specs.push(resolve_php_spec(spec)?);
        }
        payload.specs = resolved_specs;
    }

    if payload.rehash && payload.ecosystem.is_none() {
        payload.ecosystem = Some("php".to_string());
    }

    apply_flags(&mut payload, context);
    Ok(payload)
}

fn apply_flags(payload: &mut InstallPayload, context: &Context) {
    if context.args.flags.contains_key("--yes") || context.args.flags.contains_key("-y") {
        payload.yes = true;
    }
    if context.args.flags.contains_key("--prompt") || context.args.flags.contains_key("-p") {
        payload.prompt = true;
    }
    if context.args.flags.contains_key("--quiet") || context.args.flags.contains_key("-q") {
        payload.quiet = true;
    }
    if context.args.flags.contains_key("--rehash") {
        payload.rehash = true;
    }
}

fn parse_spec_list(value: &str) -> Vec<String> {
    value
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn resolve_php_spec(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(trimmed.to_string());
    }
    if trimmed.starts_with('@') {
        if is_valid_scoped_name(trimmed) {
            return Ok(trimmed.to_string());
        }
        return Err(anyhow::anyhow!(
            "invalid php package `{}` (expected @scope/name[@version])",
            trimmed
        ));
    }

    match trimmed {
        "json" | "jwt" => Ok(format!("@deka/{}", trimmed)),
        _ => Err(anyhow::anyhow!(
            "unscoped php package `{}` is not allowed. use @scope/name (or stdlib alias like json/jwt)",
            trimmed
        )),
    }
}

fn is_valid_scoped_name(spec: &str) -> bool {
    let without_version = if let Some(idx) = spec[1..].find('@') {
        &spec[..idx + 1]
    } else {
        spec
    };
    let mut parts = without_version.split('/');
    let scope = parts.next().unwrap_or("");
    let name = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return false;
    }
    if !scope.starts_with('@') || scope.len() <= 1 || name.is_empty() {
        return false;
    }
    scope[1..]
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}
