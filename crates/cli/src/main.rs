use core::Registry;
use r#loop as loop_cmd;
use wasm_cli as wasm_cmd;

mod cli;

fn main() {
    let mut registry = Registry::new();
    cli::register_global_flags(&mut registry);
    cli::register_global_params(&mut registry);
    cli::build::register(&mut registry);
    cli::compile::register(&mut registry);
    cli::db::register(&mut registry);
    cli::init::register(&mut registry);
    cli::install::register(&mut registry);
    cli::lsp::register(&mut registry);
    cli::publish::register(&mut registry);
    cli::run::register(&mut registry);
    cli::serve::register(&mut registry);
    cli::self_cmd::register(&mut registry);
    cli::test::register(&mut registry);
    cli::user::register(&mut registry);
    wasm_cmd::register(&mut registry);
    introspect::register(&mut registry);
    loop_cmd::register(&mut registry);

    cli::execute(&registry);
}
