use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    Term,
    Kill,
    Int,
    Custom(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdioMode {
    Inherit,
    Piped,
    Null,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SpawnOptions {
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
    pub clear_env: bool,
    pub stdin: StdioMode,
    pub stdout: StdioMode,
    pub stderr: StdioMode,
    pub pty: bool,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            env: BTreeMap::new(),
            clear_env: false,
            stdin: StdioMode::Piped,
            stdout: StdioMode::Piped,
            stderr: StdioMode::Piped,
            pty: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus {
    pub code: i32,
    pub signal: Option<ProcessSignal>,
}

pub trait ReadableStream: Send {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn close(&mut self) -> Result<()>;
}

pub trait WritableStream: Send {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

pub trait ProcessHandle: Send {
    fn id(&self) -> ProcessId;
    fn stdin(&mut self) -> Option<&mut dyn WritableStream>;
    fn stdout(&mut self) -> Option<&mut dyn ReadableStream>;
    fn stderr(&mut self) -> Option<&mut dyn ReadableStream>;
    fn wait(&mut self) -> Result<ExitStatus>;
    fn kill(&mut self, signal: ProcessSignal) -> Result<()>;
}

pub trait ProcessHost: Send + Sync {
    fn spawn(&self, command: Command, options: SpawnOptions) -> Result<Box<dyn ProcessHandle>>;
}
