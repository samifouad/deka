pub mod init;

#[derive(Debug, Clone)]
pub struct Args {
    pub flags: std::collections::HashMap<String, bool>,
    pub params: std::collections::HashMap<String, String>,
    pub commands: Vec<String>,
}

pub fn collect(args: Vec<String>) -> Args {
    let mut flags = std::collections::HashMap::new();
    let mut params = std::collections::HashMap::new();
    let mut commands = Vec::new();

    let valid_flags = [
        "--help",
        "help",
        "-H",
        "--version",
        "version",
        "-V",
        "--update",
        "update",
        "-U",
        "--watch",
        "watch",
        "-W",
    ];

    let valid_params = ["--port", "--mode", "--folder"];

    let valid_commands = ["start", "tree", "route", "init", "build", "deploy"];

    let mut iter = args.iter().enumerate();
    while let Some((_i, arg)) = iter.next() {
        if valid_flags.contains(&arg.as_str()) {
            flags.insert(arg.clone(), true);
        } else if valid_params.contains(&arg.as_str()) {
            if let Some(value) = iter.next().map(|(_, v)| v) {
                params.insert(arg.clone(), value.clone());
            } else {
                println!("Error: '{}' requires a value.", arg);
            }
        } else if valid_commands.contains(&arg.as_str()) {
            commands.push(arg.clone());
        }
    }

    Args {
        flags,
        params,
        commands,
    }
}

// provide helpful info if no args are provided
pub fn help() {
    println!("deka [version {}]\n", env!("CARGO_PKG_VERSION"));
    println!("usage: deka [--version] [--help] [--update] <command> [<params>]\n");
    println!("commands:\n");
    println!("start\t\tstarts the server");
    println!("init\t\tinitialize a new app project");
    println!("tree\t\tdisplay detected routes in tree format");
    println!("route\t\tinspect a route in your project");
    println!("build\t\toptimizes your project for production");
    println!("deploy\t\tdeploys your project to a remote server");
    println!("\n");
}

pub fn version() {
    println!(
        "deka [version {}]\n\nto check for updates run: deka --update\n",
        env!("CARGO_PKG_VERSION")
    );
}

pub fn update() {
    println!("this will check for updates and offer the ability to run the update. not yet implemented. \n");
}

pub fn error(msg: Option<&String>) {
    println!(
        "{}",
        msg.unwrap_or(&"\ninstructions unclear. try '--help' for guidance\n".to_string())
    );
}