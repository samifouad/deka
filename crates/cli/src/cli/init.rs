use core::{CommandSpec, Context, Registry};

const COMMAND: CommandSpec = CommandSpec {
    name: "init",
    category: "project",
    summary: "initialize a new app project",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

pub fn cmd(context: &Context) {
    for (key, value) in context.args.params.iter() {
        println!("Key: {}, Value: {}", key, value);
    }
    //println!("initializing new project folder at {}", path.unwrap_or(&"./app".to_string()));
}
