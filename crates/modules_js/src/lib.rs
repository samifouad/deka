use deno_core::Extension;

pub mod modules;
pub mod redis_client;

pub fn extensions() -> Vec<Extension> {
    vec![
        modules::deka::init(),
        modules::postgres::register_ops(),
        modules::docker::register_ops(),
        modules::router::init(),
        modules::t4::register_ops(),
        modules::sqlite::register_ops(),
    ]
}

pub fn static_extensions() -> Vec<Extension> {
    vec![modules::deka_static::init()]
}
