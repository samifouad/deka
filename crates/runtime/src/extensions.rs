use deno_core::Extension;
use engine::config::ServeMode;

use modules_common::permissions_extension;
use modules_js;
use modules_php;

pub(crate) fn extensions_for_mode(mode: &ServeMode) -> Vec<Extension> {
    let mut extensions = vec![
        permissions_extension(),
        deno_napi::deno_napi::init_ops::<deno_permissions::PermissionsContainer>(),
    ];

    match mode {
        ServeMode::Js => {
            extensions.extend(modules_js::extensions());
        }
        ServeMode::Php => {
            extensions.extend(modules_php::extensions());
        }
        ServeMode::Static => {
            extensions.extend(modules_js::static_extensions());
        }
    }

    extensions
}

pub(crate) fn extensions_for_build() -> Vec<Extension> {
    extensions_for_mode(&ServeMode::Js)
}
