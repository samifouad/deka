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

struct ChildProcessEntry {
    child: Option<Child>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    stdin: Option<ChildStdin>,
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
) -> Result<u64, CoreError> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env_vars) = env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
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

    let entry = ChildProcessEntry {
        stdout: child.stdout.take(),
        stderr: child.stderr.take(),
        stdin: child.stdin.take(),
        child: Some(child),
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
) -> Result<u64, CoreError> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    if let Some(env_vars) = env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
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

    let entry = ChildProcessEntry {
        stdout: child.stdout.take(),
        stderr: child.stderr.take(),
        stdin: child.stdin.take(),
        child: Some(child),
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
