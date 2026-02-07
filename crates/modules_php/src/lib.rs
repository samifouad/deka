use deno_core::Extension;

pub mod compiler_api;
pub mod modules;
pub mod validation;

pub fn php_extension() -> Extension {
    modules::php::init()
}

pub fn extensions() -> Vec<Extension> {
    vec![modules::deka_php::init(), modules::php::init()]
}
