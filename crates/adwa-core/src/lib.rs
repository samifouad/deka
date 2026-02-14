mod error;
mod fs;
mod fs_memory;
mod host;
mod net;
mod net_memory;
mod process;
mod process_memory;

pub use error::{ErrorCode, Result, AdwaError};
pub use fs::{
    DirEntry, FileSystem, FileType, FsEvent, FsEventKind, FsWatcher, Metadata, MkdirOptions,
    MountFile, MountTree, RemoveOptions, WatchOptions, WriteOptions,
};
pub use fs_memory::InMemoryFileSystem;
pub use host::Host;
pub use net::{NetHost, PortEvent, PortInfo, PortProtocol, PortPublishOptions};
pub use net_memory::InMemoryNetHost;
pub use process::{
    Command, ExitStatus, ProcessHandle, ProcessHost, ProcessId, ProcessSignal, ReadableStream,
    SpawnOptions, StdioMode, WritableStream,
};
pub use process_memory::InMemoryProcessHost;

/// Marker type for the core runtime surface.
#[derive(Debug, Default)]
pub struct AdwaCore;

impl AdwaCore {
    pub fn new() -> Self {
        Self
    }
}
