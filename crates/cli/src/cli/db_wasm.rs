use core::{CommandSpec, Context, Registry, SubcommandSpec};
use stdio::{error, log};

const GENERATE: SubcommandSpec = SubcommandSpec {
    name: "generate",
    summary: "generate db client and migration artifacts (browser platform status)",
    aliases: &["gen"],
    handler: cmd_generate,
};

const MIGRATE: SubcommandSpec = SubcommandSpec {
    name: "migrate",
    summary: "apply pending db migrations (browser platform status)",
    aliases: &[],
    handler: cmd_migrate,
};

const INFO: SubcommandSpec = SubcommandSpec {
    name: "info",
    summary: "show db generation and migration status (browser platform status)",
    aliases: &["status"],
    handler: cmd_info,
};

const FLUSH: SubcommandSpec = SubcommandSpec {
    name: "flush",
    summary: "reset database schema (browser platform status)",
    aliases: &[],
    handler: cmd_flush,
};

const SUBCOMMANDS: &[SubcommandSpec] = &[GENERATE, MIGRATE, INFO, FLUSH];

const COMMAND: CommandSpec = CommandSpec {
    name: "db",
    category: "database",
    summary: "database tooling (platform-gated)",
    aliases: &[],
    subcommands: SUBCOMMANDS,
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

fn cmd(_context: &Context) {
    error(
        "db",
        "missing subcommand. use: deka db generate|migrate|info|flush",
    );
}

fn cmd_generate(_context: &Context) {
    log(
        "db generate",
        "unavailable on platform_browser today; run on platform_server for now",
    );
}

fn cmd_migrate(_context: &Context) {
    log(
        "db migrate",
        "unavailable on platform_browser today; run on platform_server for now",
    );
}

fn cmd_info(_context: &Context) {
    log(
        "db info",
        "unavailable on platform_browser today; run on platform_server for now",
    );
}

fn cmd_flush(_context: &Context) {
    log(
        "db flush",
        "unavailable on platform_browser today; run on platform_server for now",
    );
}
