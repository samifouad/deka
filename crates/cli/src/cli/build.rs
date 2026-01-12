use core::{CommandSpec, Context, FlagSpec, Registry};

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
}

pub fn cmd(context: &Context) {
    runtime::build(context);
}
