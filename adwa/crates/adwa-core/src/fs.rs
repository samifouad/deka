use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub file_type: FileType,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub file_type: FileType,
}

#[derive(Debug, Clone)]
pub struct WriteOptions {
    pub create: bool,
    pub truncate: bool,
    pub mode: Option<u32>,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            create: true,
            truncate: true,
            mode: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MkdirOptions {
    pub recursive: bool,
    pub mode: Option<u32>,
}

impl Default for MkdirOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            mode: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoveOptions {
    pub recursive: bool,
    pub force: bool,
}

impl Default for RemoveOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            force: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MountFile {
    pub data: Vec<u8>,
    pub executable: bool,
}

#[derive(Debug, Clone)]
pub enum MountTree {
    File(MountFile),
    Directory(BTreeMap<String, MountTree>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEventKind {
    Created,
    Modified,
    Removed,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct FsEvent {
    pub path: String,
    pub kind: FsEventKind,
    pub target_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WatchOptions {
    pub recursive: bool,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self { recursive: true }
    }
}

pub trait FsWatcher: Send {
    fn next_event(&mut self) -> Result<Option<FsEvent>>;
    fn close(&mut self) -> Result<()>;
}

pub trait FileSystem: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    fn write_file(&self, path: &Path, data: &[u8], options: WriteOptions) -> Result<()>;
    fn mkdir(&self, path: &Path, options: MkdirOptions) -> Result<()>;
    fn readdir(&self, path: &Path) -> Result<Vec<DirEntry>>;
    fn stat(&self, path: &Path) -> Result<Metadata>;
    fn remove(&self, path: &Path, options: RemoveOptions) -> Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn mount_tree(&self, path: &Path, tree: MountTree) -> Result<()>;
    fn watch(&self, path: &Path, options: WatchOptions) -> Result<Box<dyn FsWatcher>>;
}
