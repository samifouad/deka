use core::Context;
use compile::vfs::RuntimeMode;

mod build;
mod env;
mod extensions;
mod run;
mod serve;
mod vfs_loader;
mod serve_vfs;
mod desktop;

pub fn run(context: &Context) {
    // Check for VFS mode
    if let Some(_vfs) = vfs_loader::detect_embedded_vfs() {
        // TODO: Implement run_vfs
        eprintln!("[vfs] VFS detected but run_vfs not yet implemented");
        run::run(context);
    } else {
        run::run(context);
    }
}

pub fn build(context: &Context) {
    build::build(context);
}

pub fn serve(context: &Context) {
    // Check for VFS mode
    if let Some(vfs) = vfs_loader::detect_embedded_vfs() {
        // Check if this is a desktop app
        if vfs.mode() == &RuntimeMode::Desktop {
            desktop::serve_desktop(context, vfs);
        } else {
            serve_vfs::serve_vfs(context, vfs);
        }
    } else {
        serve::serve(context);
    }
}

pub fn serve_desktop(context: &Context) {
    // Check for VFS mode (required for desktop)
    if let Some(vfs) = vfs_loader::detect_embedded_vfs() {
        desktop::serve_desktop(context, vfs);
    } else {
        eprintln!("[desktop] Desktop mode requires a compiled binary with embedded VFS");
        std::process::exit(1);
    }
}

pub fn has_embedded_vfs() -> bool {
    vfs_loader::detect_embedded_vfs().is_some()
}
