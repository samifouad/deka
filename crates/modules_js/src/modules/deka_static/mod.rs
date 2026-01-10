// Minimal static file serving prelude module
use deno_core::Extension;

deno_core::extension!(
    deka_static,
    esm_entry_point = "ext:deka_static/static.js",
    esm = [dir "src/modules/deka_static", "static.js"],
);

pub fn init() -> Extension {
    deka_static::init_ops_and_esm()
}
