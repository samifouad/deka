use deno_core::Extension;

pub mod compiler_api;
pub mod integrity;
pub mod modules;
pub mod validation;

pub fn php_extension() -> Extension {
    modules::php::init()
}

pub fn extensions() -> Vec<Extension> {
    // MVP2 JS runtime path: keep php_core loaded for host ops/prelude glue.
    // Legacy deka_php wasm runtime remains in-tree but is no longer wired.
    vec![modules::php::init()]
}
