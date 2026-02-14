use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use crate::{
    Command, ErrorCode, ExitStatus, FileSystem, MkdirOptions, ProcessHandle, ProcessHost,
    ProcessId, ProcessSignal, ReadableStream, RemoveOptions, Result, SpawnOptions, StdioMode,
    AdwaError, WritableStream, WriteOptions,
};

#[derive(Default)]
pub struct InMemoryProcessHost {
    next_id: AtomicU32,
    fs: Option<Arc<dyn FileSystem>>,
}

impl InMemoryProcessHost {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU32::new(1),
            fs: None,
        }
    }

    pub fn with_fs(fs: Arc<dyn FileSystem>) -> Self {
        Self {
            next_id: AtomicU32::new(1),
            fs: Some(fs),
        }
    }
}

impl ProcessHost for InMemoryProcessHost {
    fn spawn(&self, command: Command, options: SpawnOptions) -> Result<Box<dyn ProcessHandle>> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let cwd = options.cwd.as_deref().unwrap_or_else(|| Path::new("/"));
        let output = execute_command(&command, cwd, self.fs.as_deref())?;
        Ok(Box::new(InMemoryProcess::new(
            ProcessId(id),
            options,
            output,
        )))
    }
}

#[derive(Debug)]
struct InMemoryProcess {
    id: ProcessId,
    exit_status: ExitStatus,
    running: bool,
    stdin: Option<MemoryWritable>,
    stdout: Option<MemoryReadable>,
    stderr: Option<MemoryReadable>,
}

impl InMemoryProcess {
    fn new(id: ProcessId, options: SpawnOptions, output: CommandOutput) -> Self {
        let stdin = match options.stdin {
            StdioMode::Piped => Some(MemoryWritable::new()),
            _ => None,
        };
        let stdout = match options.stdout {
            StdioMode::Piped => Some(MemoryReadable::from_bytes(output.stdout)),
            _ => None,
        };
        let stderr = match options.stderr {
            StdioMode::Piped => Some(MemoryReadable::from_bytes(output.stderr)),
            _ => None,
        };
        Self {
            id,
            exit_status: output.status,
            running: output.keep_alive,
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
        self.stdin
            .as_mut()
            .map(|stream| stream as &mut dyn WritableStream)
    }

    fn stdout(&mut self) -> Option<&mut dyn crate::ReadableStream> {
        self.stdout
            .as_mut()
            .map(|stream| stream as &mut dyn ReadableStream)
    }

    fn stderr(&mut self) -> Option<&mut dyn crate::ReadableStream> {
        self.stderr
            .as_mut()
            .map(|stream| stream as &mut dyn ReadableStream)
    }

    fn wait(&mut self) -> Result<ExitStatus> {
        if self.running {
            return Err(AdwaError::new(ErrorCode::Busy, "process still running"));
        }
        Ok(self.exit_status)
    }

    fn kill(&mut self, signal: ProcessSignal) -> Result<()> {
        self.running = false;
        self.exit_status = ExitStatus {
            code: 128,
            signal: Some(signal),
        };
        Ok(())
    }
}

#[derive(Debug)]
struct CommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    keep_alive: bool,
}

impl CommandOutput {
    fn ok(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            status: ExitStatus {
                code: 0,
                signal: None,
            },
            stdout: stdout.into(),
            stderr: Vec::new(),
            keep_alive: false,
        }
    }

    fn fail(code: i32, stderr: impl Into<Vec<u8>>) -> Self {
        Self {
            status: ExitStatus { code, signal: None },
            stdout: Vec::new(),
            stderr: stderr.into(),
            keep_alive: false,
        }
    }

    fn long_running(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            status: ExitStatus {
                code: 0,
                signal: None,
            },
            stdout: stdout.into(),
            stderr: Vec::new(),
            keep_alive: true,
        }
    }
}

fn execute_command(
    command: &Command,
    cwd: &Path,
    fs: Option<&dyn FileSystem>,
) -> Result<CommandOutput> {
    let program = command.program.trim();
    let args = command.args.as_slice();

    match program {
        "help" => Ok(CommandOutput::ok(
            b"commands: help, pwd, ls [path], cat <file>, echo [text], mkdir <dir>, touch <file>, cp <src> <dst>, mv <src> <dst>, rm <path>, deka run <file>\n".to_vec(),
        )),
        "pwd" => Ok(CommandOutput::ok(
            format!("{}\n", normalize_display_path(cwd)).into_bytes(),
        )),
        "echo" => Ok(CommandOutput::ok(
            format!("{}\n", args.join(" ")).into_bytes(),
        )),
        "ls" => run_ls(cwd, args, fs),
        "cat" => run_cat(cwd, args, fs),
        "mkdir" => run_mkdir(cwd, args, fs),
        "touch" => run_touch(cwd, args, fs),
        "cp" => run_cp(cwd, args, fs),
        "mv" => run_mv(cwd, args, fs),
        "rm" => run_rm(cwd, args, fs),
        "deka" => run_deka(cwd, args, fs),
        "phpx" => run_phpx(cwd, args, fs),
        other => Ok(CommandOutput::fail(
            127,
            format!("unknown command: {other}\n").into_bytes(),
        )),
    }
}

fn run_deka(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    if args.is_empty() {
        return Ok(CommandOutput::fail(
            2,
            b"deka: missing subcommand\n".to_vec(),
        ));
    }
    if args[0] == "run" {
        let file = args.get(1).map(String::as_str).unwrap_or("main.phpx");
        let path = resolve_path(cwd, file);
        return match fs {
            Some(fs) => match fs.read_file(&path) {
                Ok(_) => Ok(CommandOutput::ok(
                    format!("deka run stub: {}\n", path.display()).into_bytes(),
                )),
                Err(_) => Ok(CommandOutput::fail(
                    1,
                    format!("deka run: file not found: {}\n", path.display()).into_bytes(),
                )),
            },
            None => Ok(CommandOutput::fail(
                1,
                b"deka run unavailable: filesystem is not mounted\n".to_vec(),
            )),
        };
    }
    if args[0] == "serve" {
        let mut entry = "main.phpx".to_string();
        let mut port: u16 = 8530;
        let mut mode = "php".to_string();

        let mut i = 1usize;
        while i < args.len() {
            let token = args[i].as_str();
            match token {
                "--port" => {
                    if let Some(next) = args.get(i + 1) {
                        if let Ok(parsed) = next.parse::<u16>() {
                            port = parsed;
                        }
                        i += 1;
                    }
                }
                "--mode" => {
                    if let Some(next) = args.get(i + 1) {
                        mode = next.clone();
                        i += 1;
                    }
                }
                _ => {
                    if !token.starts_with('-') {
                        entry = token.to_string();
                    }
                }
            }
            i += 1;
        }

        let path = resolve_path(cwd, &entry);
        if let Some(fs) = fs {
            if fs.read_file(&path).is_err() {
                return Ok(CommandOutput::fail(
                    1,
                    format!("deka serve: file not found: {}\n", path.display()).into_bytes(),
                ));
            }
        }

        let banner = format!(
            "[handler] loaded {} [mode={}]\n[listen] http://localhost:{}\n",
            path.display(),
            mode,
            port
        );
        return Ok(CommandOutput::long_running(banner.into_bytes()));
    }
    Ok(CommandOutput::fail(
        2,
        format!("deka: unsupported subcommand '{}'\n", args[0]).into_bytes(),
    ))
}

fn run_phpx(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    if args.first().map(String::as_str) != Some("run") {
        return Ok(CommandOutput::fail(
            2,
            b"phpx: supported usage is 'phpx run <file>'\n".to_vec(),
        ));
    }
    run_deka(cwd, args, fs)
}

fn run_ls(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"ls unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let target = args.first().map(String::as_str).unwrap_or(".");
    let path = resolve_path(cwd, target);
    match fs.readdir(&path) {
        Ok(entries) => {
            let names = entries
                .into_iter()
                .map(|entry| entry.name)
                .collect::<Vec<_>>()
                .join("  ");
            let body = if names.is_empty() {
                "(empty)\n".to_string()
            } else {
                format!("{names}\n")
            };
            Ok(CommandOutput::ok(body.into_bytes()))
        }
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("ls: {}\n", err.message).into_bytes(),
        )),
    }
}

fn run_cat(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"cat unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let Some(file) = args.first() else {
        return Ok(CommandOutput::fail(2, b"cat: missing file path\n".to_vec()));
    };
    let path = resolve_path(cwd, file);
    match fs.read_file(&path) {
        Ok(bytes) => {
            let mut out = bytes;
            if out.last().copied() != Some(b'\n') {
                out.push(b'\n');
            }
            Ok(CommandOutput::ok(out))
        }
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("cat: {}\n", err.message).into_bytes(),
        )),
    }
}

fn run_mkdir(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"mkdir unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let Some(path) = args.first() else {
        return Ok(CommandOutput::fail(2, b"mkdir: missing path\n".to_vec()));
    };
    let target = resolve_path(cwd, path);
    match fs.mkdir(
        &target,
        MkdirOptions {
            recursive: true,
            mode: None,
        },
    ) {
        Ok(()) => Ok(CommandOutput::ok(Vec::new())),
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("mkdir: {}\n", err.message).into_bytes(),
        )),
    }
}

fn run_touch(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"touch unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let Some(path) = args.first() else {
        return Ok(CommandOutput::fail(2, b"touch: missing path\n".to_vec()));
    };
    let target = resolve_path(cwd, path);
    match fs.write_file(
        &target,
        &[],
        WriteOptions {
            create: true,
            truncate: false,
            mode: None,
        },
    ) {
        Ok(()) => Ok(CommandOutput::ok(Vec::new())),
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("touch: {}\n", err.message).into_bytes(),
        )),
    }
}

fn run_cp(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"cp unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let Some(src) = args.first() else {
        return Ok(CommandOutput::fail(2, b"cp: missing source\n".to_vec()));
    };
    let Some(dst) = args.get(1) else {
        return Ok(CommandOutput::fail(2, b"cp: missing destination\n".to_vec()));
    };
    let src_path = resolve_path(cwd, src);
    let dst_path = resolve_path(cwd, dst);
    match fs.read_file(&src_path) {
        Ok(bytes) => match fs.write_file(
            &dst_path,
            &bytes,
            WriteOptions {
                create: true,
                truncate: true,
                mode: None,
            },
        ) {
            Ok(()) => Ok(CommandOutput::ok(Vec::new())),
            Err(err) => Ok(CommandOutput::fail(
                1,
                format!("cp: {}\n", err.message).into_bytes(),
            )),
        },
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("cp: {}\n", err.message).into_bytes(),
        )),
    }
}

fn run_mv(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"mv unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let Some(src) = args.first() else {
        return Ok(CommandOutput::fail(2, b"mv: missing source\n".to_vec()));
    };
    let Some(dst) = args.get(1) else {
        return Ok(CommandOutput::fail(2, b"mv: missing destination\n".to_vec()));
    };
    let src_path = resolve_path(cwd, src);
    let dst_path = resolve_path(cwd, dst);
    match fs.rename(&src_path, &dst_path) {
        Ok(()) => Ok(CommandOutput::ok(Vec::new())),
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("mv: {}\n", err.message).into_bytes(),
        )),
    }
}

fn run_rm(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"rm unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };
    let Some(path) = args.first() else {
        return Ok(CommandOutput::fail(2, b"rm: missing path\n".to_vec()));
    };
    let target = resolve_path(cwd, path);
    match fs.remove(
        &target,
        RemoveOptions {
            recursive: true,
            force: false,
        },
    ) {
        Ok(()) => Ok(CommandOutput::ok(Vec::new())),
        Err(err) => Ok(CommandOutput::fail(
            1,
            format!("rm: {}\n", err.message).into_bytes(),
        )),
    }
}

fn resolve_path(cwd: &Path, input: &str) -> PathBuf {
    let input = input.trim();
    if input.is_empty() || input == "." {
        return normalize_path(cwd.to_path_buf());
    }
    if input.starts_with('/') {
        return normalize_path(PathBuf::from(input));
    }
    normalize_path(cwd.join(input))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut stack: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::RootDir => {
                stack.clear();
            }
            std::path::Component::ParentDir => {
                let _ = stack.pop();
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => {
                stack.push(name.to_string_lossy().to_string());
            }
            std::path::Component::Prefix(_) => {}
        }
    }
    let mut out = PathBuf::from("/");
    for part in stack {
        out.push(part);
    }
    out
}

fn normalize_display_path(path: &Path) -> String {
    let normalized = normalize_path(path.to_path_buf());
    normalized.to_string_lossy().to_string()
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
    fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut pipe = MemoryPipe::default();
        pipe.buffer = bytes.into_iter().collect();
        Self {
            pipe: Arc::new(Mutex::new(pipe)),
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

fn stream_closed() -> AdwaError {
    AdwaError::new(ErrorCode::InvalidInput, "stream is closed")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        Command, ErrorCode, FileSystem, InMemoryFileSystem, InMemoryProcessHost, ProcessHost,
        ProcessSignal, SpawnOptions, StdioMode, WriteOptions,
    };

    #[test]
    fn deka_serve_stays_running_until_killed() {
        let fs = Arc::new(InMemoryFileSystem::new());
        fs.write_file(
            std::path::Path::new("/main.phpx"),
            b"<?php echo 'ok';",
            WriteOptions {
                create: true,
                truncate: true,
                mode: None,
            },
        )
        .expect("write test file");

        let host = InMemoryProcessHost::with_fs(fs);
        let mut handle = host
            .spawn(
                Command {
                    program: "deka".to_string(),
                    args: vec!["serve".to_string(), "main.phpx".to_string()],
                },
                SpawnOptions {
                    stdout: StdioMode::Piped,
                    stderr: StdioMode::Piped,
                    ..SpawnOptions::default()
                },
            )
            .expect("spawn deka serve");

        let mut buf = vec![0u8; 4096];
        let stdout = handle
            .stdout()
            .expect("stdout stream")
            .read(&mut buf)
            .expect("read stdout");
        let banner = String::from_utf8_lossy(&buf[..stdout]).to_string();
        assert!(banner.contains("[listen] http://localhost:8530"));

        let running = handle.wait().expect_err("serve should still be running");
        assert_eq!(running.code, ErrorCode::Busy);

        handle
            .kill(ProcessSignal::Int)
            .expect("kill running serve process");
        let exited = handle.wait().expect("wait after kill");
        assert_eq!(exited.code, 128);
        assert_eq!(exited.signal, Some(ProcessSignal::Int));
    }
}
