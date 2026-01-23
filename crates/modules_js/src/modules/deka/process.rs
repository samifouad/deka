use std::collections::HashMap;
use std::io::Write;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use deno_core::{JsBuffer, error::CoreError, op2};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex as AsyncMutex;

/// IPC channel for bidirectional message passing between parent and child
struct IpcChannel {
    socket: tokio::net::UnixStream,  // Bidirectional Unix socket for IPC
}

struct ChildProcessEntry {
    child: Option<Child>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    stdin: Option<ChildStdin>,
    ipc: Option<IpcChannel>,  // NEW: IPC channel
}

static CHILD_PROCESSES: OnceLock<Mutex<HashMap<u64, Arc<AsyncMutex<ChildProcessEntry>>>>> =
    OnceLock::new();
static CHILD_IDS: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize)]
struct ProcessReadResult {
    data: Vec<u8>,
    eof: bool,
}

#[derive(Serialize)]
struct ProcessSpawnSyncResult {
    status: i64,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[op2(fast)]
pub(super) fn op_process_exit(#[smi] code: i32) {
    std::process::exit(code);
}

#[op2]
#[bigint]
pub(super) fn op_process_spawn_immediate(
    #[string] command: String,
    #[serde] args: Vec<String>,
    #[string] cwd: Option<String>,
    #[serde] env: Option<Vec<(String, String)>>,
    #[string] stdio: Option<String>,
    enable_ipc: bool,  // NEW: Enable IPC channel
) -> Result<u64, CoreError> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    // Create IPC socket pair if needed (Unix only)
    #[cfg(unix)]
    let ipc_socket_pair = if enable_ipc {
        Some(tokio::net::UnixStream::pair().map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create IPC socket pair: {}", err),
            ))
        })?)
    } else {
        None
    };

    #[cfg(not(unix))]
    let ipc_socket_pair: Option<(tokio::net::UnixStream, tokio::net::UnixStream)> = None;

    if let Some(env_vars) = env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    // Pass IPC file descriptor to child - dup2 it to fd 3 in the child using pre_exec
    #[cfg(unix)]
    if let Some((_, ref child_socket)) = ipc_socket_pair {
        use std::os::unix::io::AsRawFd;
        let source_fd = child_socket.as_raw_fd();

        // Use pre_exec to dup2 in the child process (after fork, before exec)
        unsafe {
            cmd.pre_exec(move || {
                let target_fd = 3;
                // Dup source to target (fd 3)
                if libc::dup2(source_fd, target_fd) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // Close the original source_fd if it's not fd 3
                if source_fd != target_fd {
                    libc::close(source_fd);
                }
                // Clear close-on-exec flag on fd 3
                let flags = libc::fcntl(target_fd, libc::F_GETFD);
                if flags >= 0 {
                    libc::fcntl(target_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC);
                }
                Ok(())
            });
        }

        cmd.env("DEKA_IPC_ENABLED", "1");
        cmd.env("DEKA_IPC_FD", "3"); // Always fd 3 after dup2 in child
    }

    // Handle stdio option
    let stdio_mode = stdio.as_deref().unwrap_or("pipe");
    match stdio_mode {
        "inherit" => {
            cmd.stdin(std::process::Stdio::inherit());
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
        }
        _ => {
            // Default to pipe for any other value
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());
        }
    }

    let mut child = cmd.spawn().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Spawn failed: {}", err),
        ))
    })?;

    // Take parent socket from pair (child socket is passed via env)
    let ipc_channel = ipc_socket_pair.map(|(parent_socket, _)| IpcChannel {
        socket: parent_socket,
    });

    let entry = ChildProcessEntry {
        stdout: child.stdout.take(),
        stderr: child.stderr.take(),
        stdin: child.stdin.take(),
        child: Some(child),
        ipc: ipc_channel,
    };

    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let id = CHILD_IDS.fetch_add(1, Ordering::Relaxed);
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Child store locked",
        ))
    })?;
    guard.insert(id, Arc::new(AsyncMutex::new(entry)));
    Ok(id)
}

#[op2(async)]
#[bigint]
pub(super) async fn op_process_spawn(
    #[string] command: String,
    #[serde] args: Vec<String>,
    #[string] cwd: Option<String>,
    #[serde] env: Option<Vec<(String, String)>>,
    #[string] stdio: Option<String>,
    enable_ipc: bool,  // NEW: Enable IPC channel
) -> Result<u64, CoreError> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    // Create IPC socket pair if needed (Unix only)
    #[cfg(unix)]
    let ipc_socket_pair = if enable_ipc {
        Some(tokio::net::UnixStream::pair().map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create IPC socket pair: {}", err),
            ))
        })?)
    } else {
        None
    };

    #[cfg(not(unix))]
    let ipc_socket_pair: Option<(tokio::net::UnixStream, tokio::net::UnixStream)> = None;

    if let Some(env_vars) = env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    // Pass IPC file descriptor to child - dup2 it to fd 3 in the child using pre_exec
    #[cfg(unix)]
    if let Some((_, ref child_socket)) = ipc_socket_pair {
        use std::os::unix::io::AsRawFd;
        let source_fd = child_socket.as_raw_fd();

        // Use pre_exec to dup2 in the child process (after fork, before exec)
        unsafe {
            cmd.pre_exec(move || {
                let target_fd = 3;
                // Dup source to target (fd 3)
                if libc::dup2(source_fd, target_fd) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // Close the original source_fd if it's not fd 3
                if source_fd != target_fd {
                    libc::close(source_fd);
                }
                // Clear close-on-exec flag on fd 3
                let flags = libc::fcntl(target_fd, libc::F_GETFD);
                if flags >= 0 {
                    libc::fcntl(target_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC);
                }
                Ok(())
            });
        }

        cmd.env("DEKA_IPC_ENABLED", "1");
        cmd.env("DEKA_IPC_FD", "3"); // Always fd 3 after dup2 in child
    }

    // Handle stdio option
    let stdio_mode = stdio.as_deref().unwrap_or("pipe");
    match stdio_mode {
        "inherit" => {
            cmd.stdin(std::process::Stdio::inherit());
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
        }
        _ => {
            // Default to pipe for any other value
            cmd.stdin(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());
        }
    }

    let mut child = cmd.spawn().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Spawn failed: {}", err),
        ))
    })?;

    // Take parent socket from pair (child socket is passed via env)
    let ipc_channel = ipc_socket_pair.map(|(parent_socket, _)| IpcChannel {
        socket: parent_socket,
    });

    let entry = ChildProcessEntry {
        stdout: child.stdout.take(),
        stderr: child.stderr.take(),
        stdin: child.stdin.take(),
        child: Some(child),
        ipc: ipc_channel,
    };

    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let id = CHILD_IDS.fetch_add(1, Ordering::Relaxed);
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Child store locked",
        ))
    })?;
    guard.insert(id, Arc::new(AsyncMutex::new(entry)));
    Ok(id)
}

#[op2]
#[serde]
pub(super) fn op_process_spawn_sync(
    #[string] command: String,
    #[serde] args: Vec<String>,
    #[string] cwd: Option<String>,
    #[serde] env: Option<Vec<(String, String)>>,
    #[buffer] input: &[u8],
) -> Result<ProcessSpawnSyncResult, CoreError> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env_vars) = env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Spawn failed: {}", err),
        ))
    })?;
    if let Some(mut stdin) = child.stdin.take() {
        if !input.is_empty() {
            stdin.write_all(input).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    err.to_string(),
                ))
            })?;
        }
    }
    let output = child.wait_with_output().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    Ok(ProcessSpawnSyncResult {
        status: output.status.code().unwrap_or(-1) as i64,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

#[op2(async)]
#[serde]
pub(super) async fn op_process_read_stdout(
    #[bigint] id: u64,
    #[smi] max_bytes: u32,
) -> Result<ProcessReadResult, CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut stdout = {
        let mut entry = entry.lock().await;
        entry.stdout.take().ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "stdout closed",
            ))
        })?
    };
    let mut buf = vec![0u8; max_bytes as usize];
    let read = stdout.read(&mut buf).await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    {
        let mut entry = entry.lock().await;
        entry.stdout = Some(stdout);
    }
    buf.truncate(read);
    Ok(ProcessReadResult {
        data: buf,
        eof: read == 0,
    })
}

#[op2(async)]
#[serde]
pub(super) async fn op_process_read_stderr(
    #[bigint] id: u64,
    #[smi] max_bytes: u32,
) -> Result<ProcessReadResult, CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut stderr = {
        let mut entry = entry.lock().await;
        entry.stderr.take().ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "stderr closed",
            ))
        })?
    };
    let mut buf = vec![0u8; max_bytes as usize];
    let read = stderr.read(&mut buf).await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    {
        let mut entry = entry.lock().await;
        entry.stderr = Some(stderr);
    }
    buf.truncate(read);
    Ok(ProcessReadResult {
        data: buf,
        eof: read == 0,
    })
}

#[op2(async)]
pub(super) async fn op_process_write_stdin(
    #[bigint] id: u64,
    #[buffer] data: JsBuffer,
) -> Result<(), CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut stdin = {
        let mut entry = entry.lock().await;
        entry.stdin.take().ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "stdin closed",
            ))
        })?
    };
    stdin.write_all(&data).await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    {
        let mut entry = entry.lock().await;
        entry.stdin = Some(stdin);
    }
    Ok(())
}

#[op2(async)]
pub(super) async fn op_process_close_stdin(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut entry = entry.lock().await;
    entry.stdin = None;
    Ok(())
}

#[op2(async)]
#[bigint]
pub(super) async fn op_process_wait(#[bigint] id: u64) -> Result<i64, CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut child = {
        let mut entry = entry.lock().await;
        entry.child.take()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;
    let status = child.wait().await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    Ok(status.code().unwrap_or(-1) as i64)
}

#[op2(async)]
pub(super) async fn op_process_kill(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut entry = entry.lock().await;
    let child = entry.child.as_mut().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;
    child.kill().await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    Ok(())
}

#[op2(async)]
pub(super) async fn op_sleep(#[bigint] ms: u64) -> Result<(), CoreError> {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    Ok(())
}

#[op2(async)]
pub(super) async fn op_process_send_message(
    #[bigint] id: u64,
    #[string] message: String,
) -> Result<(), CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut entry_guard = entry.lock().await;
    let ipc = entry_guard.ipc.as_mut().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "IPC channel not available",
        ))
    })?;

    // Write message with newline delimiter (JSON Lines format)
    ipc.socket.write_all(message.as_bytes()).await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write to IPC channel: {}", err),
        ))
    })?;
    ipc.socket.write_all(b"\n").await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write newline to IPC channel: {}", err),
        ))
    })?;
    ipc.socket.flush().await.map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to flush IPC channel: {}", err),
        ))
    })?;

    Ok(())
}

#[derive(Serialize)]
struct IpcReadResult {
    message: Option<String>,
}

#[op2(async)]
#[serde]
pub(super) async fn op_process_read_message(
    #[bigint] id: u64,
) -> Result<IpcReadResult, CoreError> {
    let store = CHILD_PROCESSES.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Child store locked",
            ))
        })?;
        guard.get(&id).cloned()
    }
    .ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Process not found",
        ))
    })?;

    let mut entry_guard = entry.lock().await;
    let ipc = entry_guard.ipc.as_mut().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "IPC channel not available",
        ))
    })?;

    // Read byte-by-byte until newline (JSON Lines format)
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        match ipc.socket.read_exact(&mut byte).await {
            Ok(_) => {
                if byte[0] == b'\n' {
                    // Found newline, decode and return message
                    let message = String::from_utf8(buffer).map_err(|err| {
                        CoreError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid UTF-8 in IPC message: {}", err),
                        ))
                    })?;
                    return Ok(IpcReadResult {
                        message: Some(message),
                    });
                } else {
                    buffer.push(byte[0]);
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                // EOF - channel closed
                return Ok(IpcReadResult { message: None });
            }
            Err(err) => {
                return Err(CoreError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read from IPC channel: {}", err),
                )));
            }
        }
    }
}

// ============================================================================
// Child-side IPC ops (for use within the child process)
// ============================================================================

/// Child-side op to send a message to the parent process via IPC
#[op2(async)]
pub(super) async fn op_child_ipc_send(
    #[smi] fd: i32,
    #[string] message: String,
) -> Result<(), CoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::io::FromRawFd;

        // Duplicate the file descriptor so we can safely wrap it
        let dup_fd = unsafe { libc::dup(fd) };
        if dup_fd < 0 {
            return Err(CoreError::from(std::io::Error::last_os_error()));
        }

        // SAFETY: We just created dup_fd and we own it
        let std_stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(dup_fd) };
        std_stream.set_nonblocking(true).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set non-blocking: {}", err),
            ))
        })?;

        let mut socket = tokio::net::UnixStream::from_std(std_stream).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create tokio UnixStream: {}", err),
            ))
        })?;

        // Write message with newline delimiter (JSON Lines format)
        socket.write_all(message.as_bytes()).await.map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write to IPC channel: {}", err),
            ))
        })?;
        socket.write_all(b"\n").await.map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write newline to IPC channel: {}", err),
            ))
        })?;
        socket.flush().await.map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to flush IPC channel: {}", err),
            ))
        })?;

        Ok(())
    }

    #[cfg(not(unix))]
    {
        Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "IPC is only supported on Unix platforms",
        )))
    }
}

/// Child-side op to read a message from the parent process via IPC
#[op2(async)]
#[serde]
pub(super) async fn op_child_ipc_read(
    #[smi] fd: i32,
) -> Result<IpcReadResult, CoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::io::FromRawFd;

        // Duplicate the file descriptor so we can safely wrap it
        let dup_fd = unsafe { libc::dup(fd) };
        if dup_fd < 0 {
            return Err(CoreError::from(std::io::Error::last_os_error()));
        }

        // SAFETY: We just created dup_fd and we own it
        let std_stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(dup_fd) };
        std_stream.set_nonblocking(true).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set non-blocking: {}", err),
            ))
        })?;

        let mut socket = tokio::net::UnixStream::from_std(std_stream).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create tokio UnixStream: {}", err),
            ))
        })?;

        // Read byte-by-byte until newline (JSON Lines format)
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            match socket.read_exact(&mut byte).await {
                Ok(_) => {
                    if byte[0] == b'\n' {
                        // Found newline, decode and return message
                        let message = String::from_utf8(buffer).map_err(|err| {
                            CoreError::from(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid UTF-8 in IPC message: {}", err),
                            ))
                        })?;

                        return Ok(IpcReadResult {
                            message: Some(message),
                        });
                    } else {
                        buffer.push(byte[0]);
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // EOF - channel closed
                    return Ok(IpcReadResult { message: None });
                }
                Err(err) => {
                    return Err(CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to read from IPC channel: {}", err),
                    )));
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "IPC is only supported on Unix platforms",
        )))
    }
}
