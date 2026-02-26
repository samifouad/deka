use crate::{FileSystem, NetHost, ProcessHost};

/// Host surface that wires core logic to platform adapters.
pub trait Host: Send + Sync {
    fn fs(&self) -> &dyn FileSystem;
    fn process(&self) -> &dyn ProcessHost;
    fn net(&self) -> &dyn NetHost;
}
