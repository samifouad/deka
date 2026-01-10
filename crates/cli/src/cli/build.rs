use core::{CommandSpec, Context, Registry};

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
}

pub fn cmd(context: &Context) {
    runtime::build(context);
}
