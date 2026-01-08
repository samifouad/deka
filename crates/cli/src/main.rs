mod cli;

fn main() {
    // gather cli args
    let cmd = cli::collect(std::env::args().skip(1).collect());

    // debug
    // println!("Flags: {:?}", cmd.flags);
    // println!("Parameters: {:?}", cmd.params);
    // println!("Commands: {:?}", cmd.commands);

    // check if there are any command-line arguments provided
    if cmd.commands.is_empty() {
        // returning help if no commands or flags are provided, else check for flags that return content to user
        if cmd.flags.is_empty() {
            cli::help();
        } else {
            if cmd.flags.contains_key("--help")
                || cmd.flags.contains_key("-H")
                || cmd.flags.contains_key("help")
            {
                cli::help();
            }
            if cmd.flags.contains_key("--version")
                || cmd.flags.contains_key("-V")
                || cmd.flags.contains_key("version")
            {
                cli::version();
            }
            if cmd.flags.contains_key("--update")
                || cmd.flags.contains_key("-U")
                || cmd.flags.contains_key("update")
            {
                cli::update();
            }
        }
    } else {
        // more than 1 command, return generic error
        if (cmd.commands.len()) > 1 {
            cli::error(None);
        }

        // get command to run
        let cmd_run = &cmd.commands[0];

        // execute command and pass in context
        match cmd_run.as_str() {
            // commands
            "init" => cli::init::cmd(&cmd),

            "run" => runtime::run(&cmd),

            //"route" => cli::route::cmd(&cmd),

            //"build" => cli::build::cmd(&cmd),

            //
            // return generic error
            _ => cli::error(None),
        }
    }
}
