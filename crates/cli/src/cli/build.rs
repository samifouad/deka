use core::{CommandSpec, Context, FlagSpec, ParamSpec, Registry};

const COMMAND: CommandSpec = CommandSpec {
    name: "build",
    category: "runtime",
    summary: "build browser assets",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
    registry.add_flag(FlagSpec {
        name: "--clear-cache",
        aliases: &[],
        description: "clear the bundler cache",
    });
    registry.add_flag(FlagSpec {
        name: "--sourcemap",
        aliases: &[],
        description: "generate source maps (.js.map files)",
    });
    registry.add_flag(FlagSpec {
        name: "--minify",
        aliases: &[],
        description: "minify the output using SWC minifier",
    });
    registry.add_flag(FlagSpec {
        name: "--debug",
        aliases: &["-v", "--verbose"],
        description: "show detailed build progress and logs",
    });
    registry.add_param(ParamSpec {
        name: "--target",
        description: "build target: 'browser' (default, bundles node_modules) or 'server' (externals)",
    });
}

pub fn cmd(context: &Context) {
    runtime::build(context);
}
