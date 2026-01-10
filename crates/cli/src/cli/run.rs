use core::{CommandSpec, Context, Registry};

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
}

pub fn cmd(context: &Context) {
    runtime::run(context);
}
