use anyhow::Result;
use core::{CommandSpec, Context, FlagSpec, ParamSpec, Registry};
use pm::{InstallPayload, run_install};
use std::path::PathBuf;
use stdio;

const COMMAND: CommandSpec = CommandSpec {
    name: "install",
    category: "package",
    summary: "install dependencies via the package manager",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
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
    registry.add_param(ParamSpec {
        name: "--payload",
        description: "path to a JSON payload describing the install",
    });
    registry.add_param(ParamSpec {
        name: "--ecosystem",
        description: "ecosystem hint (node/php/gleam)",
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
            if let Err(err) = run_install(payload) {
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
    let mut payload = if let Some(payload_path) = context.args.params.get("--payload") {
        let mut payload = InstallPayload::from_file(&PathBuf::from(payload_path))?;
        if let Some(path) = context.args.params.get("--ecosystem") {
            payload.ecosystem = Some(path.clone());
        }
        payload
    } else {
        let specs = context
            .args
            .params
            .get("--spec")
            .map(|value| parse_spec_list(value))
            .unwrap_or_default();
        let ecosystem = context.args.params.get("--ecosystem").cloned();
        InstallPayload {
            specs,
            ecosystem,
            yes: false,
            prompt: false,
            quiet: false,
        }
    };

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
}

fn parse_spec_list(value: &str) -> Vec<String> {
    value
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}
