use core::Registry;
use wasm_cli as wasm_cmd;

pub mod cli;

pub fn build_registry() -> Registry {
    let mut registry = Registry::new();
    cli::register_global_flags(&mut registry);
    cli::register_global_params(&mut registry);
    cli::compile::register(&mut registry);
    cli::db::register(&mut registry);
    cli::init::register(&mut registry);
    cli::install::register(&mut registry);
    cli::lsp::register(&mut registry);
    cli::run::register(&mut registry);
    cli::serve::register(&mut registry);
    cli::self_cmd::register(&mut registry);
    cli::test::register(&mut registry);
    cli::user::register(&mut registry);
    wasm_cmd::register(&mut registry);
    introspect::register(&mut registry);
    registry
}

pub fn run() {
    let registry = build_registry();
    cli::execute(&registry);
}
