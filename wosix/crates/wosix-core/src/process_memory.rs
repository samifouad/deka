use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::{
    Command, ErrorCode, ExitStatus, ProcessHandle, ProcessHost, ProcessId, ProcessSignal,
    ReadableStream, Result, SpawnOptions, StdioMode, WritableStream, WosixError,
};

#[derive(Debug, Default)]
pub struct InMemoryProcessHost {
    next_id: AtomicU32,
}

impl InMemoryProcessHost {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(1),
        }
    }
}

impl ProcessHost for InMemoryProcessHost {
    fn spawn(&self, _command: Command, options: SpawnOptions) -> Result<Box<dyn ProcessHandle>> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        Ok(Box::new(InMemoryProcess::new(ProcessId(id), options)))
    }
}

#[derive(Debug)]
struct InMemoryProcess {
    id: ProcessId,
    exit_status: ExitStatus,
    stdin: Option<MemoryWritable>,
    stdout: Option<MemoryReadable>,
    stderr: Option<MemoryReadable>,
}

impl InMemoryProcess {
    fn new(id: ProcessId, options: SpawnOptions) -> Self {
        let stdin = match options.stdin {
            StdioMode::Piped => Some(MemoryWritable::new()),
            _ => None,
        };
        let stdout = match options.stdout {
            StdioMode::Piped => Some(MemoryReadable::new()),
            _ => None,
        };
        let stderr = match options.stderr {
            StdioMode::Piped => Some(MemoryReadable::new()),
            _ => None,
        };
        Self {
            id,
            exit_status: ExitStatus {
                code: 0,
                signal: None,
            },
            stdin,
            stdout,
            stderr,
        }
    }
}

impl ProcessHandle for InMemoryProcess {
    fn id(&self) -> ProcessId {
        self.id
    }

    fn stdin(&mut self) -> Option<&mut dyn crate::WritableStream> {
        self.stdin.as_mut().map(|stream| stream as &mut dyn WritableStream)
    }

    fn stdout(&mut self) -> Option<&mut dyn crate::ReadableStream> {
        self.stdout.as_mut().map(|stream| stream as &mut dyn ReadableStream)
    }

    fn stderr(&mut self) -> Option<&mut dyn crate::ReadableStream> {
        self.stderr.as_mut().map(|stream| stream as &mut dyn ReadableStream)
    }

    fn wait(&mut self) -> Result<ExitStatus> {
        Ok(self.exit_status)
    }

    fn kill(&mut self, signal: ProcessSignal) -> Result<()> {
        self.exit_status = ExitStatus {
            code: 128,
            signal: Some(signal),
        };
        Ok(())
    }
}

#[derive(Debug, Default)]
struct MemoryPipe {
    buffer: VecDeque<u8>,
    closed: bool,
}

#[derive(Debug)]
struct MemoryReadable {
    pipe: Arc<Mutex<MemoryPipe>>,
}

impl MemoryReadable {
    fn new() -> Self {
        Self {
            pipe: Arc::new(Mutex::new(MemoryPipe::default())),
        }
    }
}

impl ReadableStream for MemoryReadable {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut pipe = self.pipe.lock().unwrap();
        if pipe.buffer.is_empty() {
            return Ok(0);
        }
        let mut read = 0;
        while read < buf.len() {
            match pipe.buffer.pop_front() {
                Some(byte) => {
                    buf[read] = byte;
                    read += 1;
                }
                None => break,
            }
        }
        Ok(read)
    }

    fn close(&mut self) -> Result<()> {
        let mut pipe = self.pipe.lock().unwrap();
        pipe.closed = true;
        pipe.buffer.clear();
        Ok(())
    }
}

#[derive(Debug)]
struct MemoryWritable {
    pipe: Arc<Mutex<MemoryPipe>>,
}

impl MemoryWritable {
    fn new() -> Self {
        Self {
            pipe: Arc::new(Mutex::new(MemoryPipe::default())),
        }
    }
}

impl WritableStream for MemoryWritable {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut pipe = self.pipe.lock().unwrap();
        if pipe.closed {
            return Err(stream_closed());
        }
        pipe.buffer.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        let mut pipe = self.pipe.lock().unwrap();
        pipe.closed = true;
        Ok(())
    }
}

fn stream_closed() -> WosixError {
    WosixError::new(ErrorCode::InvalidInput, "stream is closed")
}
