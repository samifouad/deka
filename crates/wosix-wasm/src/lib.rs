#[cfg(feature = "web")]
mod web_container;
#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "web")]
pub use web_container::{FsHandle, FsWatchHandle, ProcessHandle, WebContainer};

/// Initialize the WASM bridge.
#[cfg_attr(feature = "web", wasm_bindgen)]
pub fn init() {
    // Placeholder for JS bridge setup.
}

/// Construct a core instance for host-side wiring.
pub fn core() -> wosix_core::WosixCore {
    wosix_core::WosixCore::new()
}
