use deno_core::Extension;
use engine::config::ServeMode;

use modules_common::permissions_extension;
use modules_php;

pub(crate) fn extensions_for_mode(mode: &ServeMode) -> Vec<Extension> {
    let _ = mode;
    let mut extensions = vec![
        permissions_extension(),
        deno_napi::deno_napi::init_ops::<deno_permissions::PermissionsContainer>(),
    ];

    extensions.extend(modules_php::extensions());

    extensions
}
