use deno_core::Extension;

pub mod modules;

pub fn php_extension() -> Extension {
    modules::php::init()
}

pub fn extensions() -> Vec<Extension> {
    vec![modules::deka_php::init(), modules::php::init()]
}
