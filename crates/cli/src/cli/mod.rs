use std::collections::BTreeMap;

use core::{Context, FlagSpec, ParamSpec, ParseError, ParseErrorKind, Registry};
use stdio::{ascii, error as stdio_error, raw};

// define & export cli's submodules
pub mod build;
pub mod compile;
pub mod db;
pub mod init;
pub mod install;
pub mod lsp;
pub mod run;
pub mod self_cmd;
pub mod serve;
pub mod test;
pub mod user;

pub fn register_global_flags(registry: &mut Registry) {
    registry.add_flag(FlagSpec {
        name: "--help",
        aliases: &["-H", "help"],
        description: "show help",
    });
    registry.add_flag(FlagSpec {
        name: "--version",
        aliases: &["-V", "version"],
        description: "show version",
    });
    registry.add_flag(FlagSpec {
        name: "--update",
        aliases: &["-U", "update"],
        description: "check for updates",
    });
    registry.add_flag(FlagSpec {
        name: "--debug",
        aliases: &["-d", "debug"],
        description: "enable debug logging",
    });
    registry.add_flag(FlagSpec {
        name: "--desktop",
        aliases: &[],
        description: "compile as desktop app (requires compile command)",
    });
    registry.add_flag(FlagSpec {
        name: "--bundle",
        aliases: &[],
        description: "create platform-native bundle (macOS: .app, Windows: installer, Linux: AppImage)",
    });
}

pub fn register_global_params(registry: &mut Registry) {
    registry.add_param(ParamSpec {
        name: "--port",
        description: "server port",
    });
    registry.add_param(ParamSpec {
        name: "--mode",
        description: "runtime mode",
    });
    registry.add_param(ParamSpec {
        name: "--folder",
        description: "target folder",
    });
    registry.add_param(ParamSpec {
        name: "--outdir",
        description: "build output directory",
    });
    registry.add_param(ParamSpec {
        name: "-o",
        description: "build output directory",
    });
    registry.add_param(ParamSpec {
        name: "--name",
        description: "app name for bundled apps",
    });
}

// provide helpful info if no args are provided
pub fn help(registry: &Registry) {
    raw(&ascii("deka"));
    raw("");
    raw("Usage: deka [options] [command]");
    raw(&format!(
        "deka v{} - the cloud is a lie",
        env!("CARGO_PKG_VERSION")
    ));
    raw("");

    let dim = "\x1b[2m";
    let reset = "\x1b[0m";

    let mut grouped: BTreeMap<&str, Vec<&core::CommandSpec>> = BTreeMap::new();
    for command in registry.commands() {
        grouped.entry(command.category).or_default().push(command);
    }

    for (category, commands) in grouped {
        raw(&format!("{dim}{category}{reset}"));
        for command in commands {
            raw(&format!("  {}\t\t{}", command.name, command.summary));
            if !command.subcommands.is_empty() {
                for subcommand in command.subcommands {
                    raw(&format!(
                        "  {} {}\t{}",
                        command.name, subcommand.name, subcommand.summary
                    ));
                }
            }
        }
        raw("");
    }

    if !registry.flags().is_empty() {
        raw(&format!("{dim}flags{reset}"));
        for flag in registry.flags() {
            raw(&format!("  {}\t\t{}", flag.name, flag.description));
        }
        raw("");
    }
}

pub fn version() {
    raw(&format!(
        "deka [version {}]\n\nto check for updates run: deka --update\n",
        env!("CARGO_PKG_VERSION")
    ));
}

pub fn update() {
    raw(
        "this will check for updates and offer the ability to run the update. not yet implemented. \n",
    );
}

pub fn error(msg: Option<&str>) {
    stdio_error(
        "cli",
        msg.unwrap_or("instructions unclear. try '--help' for guidance"),
    );
}

pub fn execute(registry: &Registry) {
    // Check for embedded VFS (compiled binary mode)
    // When a binary is compiled with VFS, it should automatically start in the appropriate mode
    if runtime::has_embedded_vfs() {
        let context = match Context::from_env(registry) {
            Ok(context) => context,
            Err(_) => {
                // For compiled binaries, create a minimal context
                let args = core::Args {
                    flags: std::collections::HashMap::new(),
                    params: std::collections::HashMap::new(),
                    commands: Vec::new(),
                    positionals: Vec::new(),
                };
                let env = core::EnvContext::load();
                let handler = match core::HandlerContext::from_env(&args) {
                    Ok(h) => h,
                    Err(_) => {
                        // Use current directory as handler
                        let resolved = core::resolve_handler_path(".").unwrap();
                        let static_config = core::StaticServeConfig::load(&resolved.directory);
                        core::HandlerContext {
                            input: ".".to_string(),
                            resolved,
                            static_config,
                            serve_config_path: None,
                            package_json_path: None,
                        }
                    }
                };
                Context { args, env, handler }
            }
        };

        // Automatically serve (which will detect desktop vs server mode from VFS)
        runtime::serve(&context);
        return;
    }

    let context = match Context::from_env(registry) {
        Ok(context) => context,
        Err(core::ContextError::Parse(errors)) => {
            let message = format_parse_errors(&errors);
            error(Some(message.as_str()));
            return;
        }
        Err(core::ContextError::HandlerResolve(message)) => {
            error(Some(message.as_str()));
            return;
        }
    };
    let cmd = &context.args;
    if cmd.flags.contains_key("--debug")
        || cmd.flags.contains_key("-d")
        || cmd.flags.contains_key("debug")
    {
        unsafe {
            std::env::set_var("DEKA_DEBUG", "1");
        }
    }

    // check if there are any command-line arguments provided
    if cmd.commands.is_empty() {
        // returning help if no commands or flags are provided, else check for flags that return content to user
        if cmd.flags.is_empty() {
            help(registry);
        } else {
            if cmd.flags.contains_key("--help")
                || cmd.flags.contains_key("-H")
                || cmd.flags.contains_key("help")
            {
                help(registry);
            }
            if cmd.flags.contains_key("--version")
                || cmd.flags.contains_key("-V")
                || cmd.flags.contains_key("version")
            {
                version();
            }
            if cmd.flags.contains_key("--update")
                || cmd.flags.contains_key("-U")
                || cmd.flags.contains_key("update")
            {
                update();
            }
        }
    } else {
        if cmd.commands.len() > 2 {
            error(None);
            return;
        }

        let cmd_name = &cmd.commands[0];
        let Some(command) = registry.command_named(cmd_name) else {
            error(None);
            return;
        };

        if cmd.commands.len() == 1 {
            (command.handler)(&context);
            return;
        }

        let sub_name = &cmd.commands[1];
        let Some(subcommand) = registry.subcommand_named(command, sub_name) else {
            error(None);
            return;
        };

        (subcommand.handler)(&context);
    }
}

pub fn format_parse_errors(errors: &[ParseError]) -> String {
    let mut output = String::new();
    for error in errors {
        match &error.kind {
            ParseErrorKind::UnknownToken => {
                output.push_str(&format!("unknown argument '{}'", error.token));
                if !error.suggestions.is_empty() {
                    output.push_str(". did you mean ");
                    output.push_str(&format_suggestions(&error.suggestions));
                    output.push('?');
                }
                output.push('\n');
            }
            ParseErrorKind::MissingParamValue { param } => {
                output.push_str(&format!("missing value for '{}'\n", param));
            }
        }
    }
    output
}

fn format_suggestions(suggestions: &[String]) -> String {
    suggestions
        .iter()
        .map(|suggestion| format!("'{}'", suggestion))
        .collect::<Vec<String>>()
        .join(", ")
}
