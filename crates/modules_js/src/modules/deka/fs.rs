use std::collections::HashMap;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::UNIX_EPOCH;

use base64::Engine;
use deno_core::{error::CoreError, op2};
use deno_error::{JsErrorBox, JsErrorClass};
use serde::{Deserialize, Serialize};

struct FileEntry {
    file: std::fs::File,
}

static FILES: OnceLock<Mutex<HashMap<u64, FileEntry>>> = OnceLock::new();
static FILE_IDS: AtomicU64 = AtomicU64::new(1);

fn file_store() -> &'static Mutex<HashMap<u64, FileEntry>> {
    FILES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn file_with_mut<T>(
    id: u64,
    f: impl FnOnce(&mut std::fs::File) -> Result<T, CoreError>,
) -> Result<T, CoreError> {
    let store = file_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "File store locked",
        ))
    })?;
    let entry = guard.get_mut(&id).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File handle not found",
        ))
    })?;
    f(&mut entry.file)
}

pub(super) fn resolve_path(path: &str) -> Result<std::path::PathBuf, CoreError> {
    if let Some(stripped) = path.strip_prefix("/ext:deka_core/").or_else(|| path.strip_prefix("ext:deka_core/")) {
        return Ok(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/modules/deka")
            .join(stripped));
    }
    if let Some(stripped) = path
        .strip_prefix("./pkg.node/")
        .or_else(|| path.strip_prefix("pkg.node/"))
    {
        return Ok(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/modules/deka/pkg.node")
            .join(stripped));
    }
    if let Some(stripped) = path.strip_prefix("./lib/").or_else(|| path.strip_prefix("lib/")) {
        return Ok(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/modules/deka/lib")
            .join(stripped));
    }
    let path = std::path::PathBuf::from(path);
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(path)
    };
    Ok(path)
}

#[derive(Serialize)]
struct FsStat {
    size: u64,
    is_file: bool,
    is_dir: bool,
    mtime_ms: Option<i64>,
}

#[derive(Serialize)]
struct FsDirEntry {
    name: String,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
}

#[derive(Debug)]
struct FsIoError {
    code: &'static str,
    message: String,
}

impl std::fmt::Display for FsIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FsIoError {}

impl JsErrorClass for FsIoError {
    fn get_class(&self) -> std::borrow::Cow<'static, str> {
        "Error".into()
    }

    fn get_message(&self) -> std::borrow::Cow<'static, str> {
        self.message.clone().into()
    }

    fn get_additional_properties(
        &self,
    ) -> Vec<(
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    )> {
        vec![("code".into(), self.code.into())]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn io_error_code(err: &std::io::Error) -> &'static str {
    match err.kind() {
        ErrorKind::NotFound => "ENOENT",
        ErrorKind::PermissionDenied => "EACCES",
        ErrorKind::AlreadyExists => "EEXIST",
        ErrorKind::InvalidInput => "EINVAL",
        ErrorKind::InvalidData => "EINVAL",
        ErrorKind::TimedOut => "ETIMEDOUT",
        ErrorKind::Interrupted => "EINTR",
        ErrorKind::WouldBlock => "EAGAIN",
        _ => "EIO",
    }
}

fn fs_io_error(path: &std::path::Path, err: std::io::Error) -> JsErrorBox {
    let code = io_error_code(&err);
    let message = format!("{}: {} ({})", code, err, path.display());
    JsErrorBox::from_err(FsIoError { code, message })
}

#[op2]
#[buffer]
pub(crate) fn op_read_file(#[string] path: String) -> Result<Vec<u8>, JsErrorBox> {
    let path = resolve_path(&path).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    std::fs::read(&path).map_err(|err| fs_io_error(&path, err))
}

#[op2(fast)]
pub(crate) fn op_fs_exists(#[string] path: String) -> Result<bool, CoreError> {
    let path = resolve_path(&path)?;
    Ok(path.exists())
}

#[op2]
#[serde]
pub(crate) fn op_fs_stat(#[string] path: String) -> Result<FsStat, JsErrorBox> {
    let path = resolve_path(&path).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let metadata = std::fs::metadata(&path).map_err(|err| fs_io_error(&path, err))?;
    let mtime_ms = metadata.modified().ok().and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .map(|dur| dur.as_millis() as i64)
    });
    Ok(FsStat {
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        mtime_ms,
    })
}

#[op2]
#[serde]
pub(crate) fn op_fs_read_dir(#[string] path: String) -> Result<Vec<FsDirEntry>, JsErrorBox> {
    let path = resolve_path(&path).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&path).map_err(|err| fs_io_error(&path, err))?;
    for entry in read_dir {
        let entry = entry.map_err(|err| fs_io_error(&path, err))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().map_err(|err| fs_io_error(&path, err))?;
        let file_type = entry.file_type().map_err(|err| fs_io_error(&path, err))?;
        entries.push(FsDirEntry {
            name,
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            is_symlink: file_type.is_symlink(),
        });
    }
    Ok(entries)
}

#[op2(fast)]
pub(crate) fn op_fs_copy_file(
    #[string] from: String,
    #[string] to: String,
) -> Result<(), JsErrorBox> {
    let from = resolve_path(&from).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    let to = resolve_path(&to).map_err(|err| JsErrorBox::generic(err.to_string()))?;
    std::fs::copy(&from, &to).map_err(|err| fs_io_error(&to, err))?;
    Ok(())
}

#[op2(fast)]
pub(crate) fn op_fs_mkdir(#[string] path: String, #[smi] recursive: i32) -> Result<(), CoreError> {
    let path = resolve_path(&path)?;
    if recursive != 0 {
        std::fs::create_dir_all(&path)
    } else {
        std::fs::create_dir(&path)
    }
    .map_err(|err| {
        CoreError::from(std::io::Error::new(
            err.kind(),
            format!("Failed to create dir {}: {}", path.display(), err),
        ))
    })?;
    Ok(())
}

#[op2(fast)]
pub(crate) fn op_fs_remove_file(#[string] path: String) -> Result<(), CoreError> {
    let path = resolve_path(&path)?;
    std::fs::remove_file(&path).map_err(|err| {
        CoreError::from(std::io::Error::new(
            err.kind(),
            format!("Failed to remove file {}: {}", path.display(), err),
        ))
    })?;
    Ok(())
}

#[op2(fast)]
pub(crate) fn op_fs_append(
    #[string] path: String,
    #[string] contents: String,
) -> Result<(), CoreError> {
    let path = resolve_path(&path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory {}: {}", parent.display(), err),
            ))
        })?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                err.kind(),
                format!("Failed to open {}: {}", path.display(), err),
            ))
        })?;
    file.write_all(contents.as_bytes()).map_err(|err| {
        CoreError::from(std::io::Error::new(
            err.kind(),
            format!("Failed to append {}: {}", path.display(), err),
        ))
    })?;
    Ok(())
}

#[op2(fast)]
pub(crate) fn op_fs_append_bytes(
    #[string] path: String,
    #[buffer] contents: &[u8],
) -> Result<(), CoreError> {
    let path = resolve_path(&path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory {}: {}", parent.display(), err),
            ))
        })?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                err.kind(),
                format!("Failed to open {}: {}", path.display(), err),
            ))
        })?;
    file.write_all(contents).map_err(|err| {
        CoreError::from(std::io::Error::new(
            err.kind(),
            format!("Failed to append {}: {}", path.display(), err),
        ))
    })?;
    Ok(())
}

#[derive(Deserialize)]
struct FsOpenOptions {
    read: bool,
    write: bool,
    append: bool,
    create: bool,
    truncate: bool,
}

#[op2]
#[bigint]
pub(crate) fn op_fs_open(
    #[string] path: String,
    #[serde] options: FsOpenOptions,
) -> Result<u64, CoreError> {
    let path = resolve_path(&path)?;
    let mut open = std::fs::OpenOptions::new();
    open.read(options.read)
        .write(options.write)
        .append(options.append)
        .create(options.create)
        .truncate(options.truncate);
    let file = open.open(&path).map_err(|err| {
        CoreError::from(std::io::Error::new(
            err.kind(),
            format!("Failed to open {}: {}", path.display(), err),
        ))
    })?;
    let id = FILE_IDS.fetch_add(1, Ordering::Relaxed);
    let store = file_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "File store locked",
        ))
    })?;
    guard.insert(id, FileEntry { file });
    Ok(id)
}

#[op2(fast)]
pub(crate) fn op_fs_close(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = file_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "File store locked",
        ))
    })?;
    guard.remove(&id);
    Ok(())
}

#[derive(Serialize)]
struct FsReadResult {
    bytes_read: u64,
    data: Vec<u8>,
}

#[op2]
#[serde]
pub(crate) fn op_fs_read(
    #[bigint] id: u64,
    #[smi] length: i32,
    #[smi] position: i64,
) -> Result<FsReadResult, CoreError> {
    let length = length.max(0) as usize;
    file_with_mut(id, |file| {
        if position >= 0 {
            file.seek(SeekFrom::Start(position as u64)).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    err.kind(),
                    format!("Failed to seek: {}", err),
                ))
            })?;
        }
        let mut buffer = vec![0u8; length];
        let read = file.read(&mut buffer).map_err(|err| {
            CoreError::from(std::io::Error::new(
                err.kind(),
                format!("Failed to read file: {}", err),
            ))
        })?;
        buffer.truncate(read);
        Ok(FsReadResult {
            bytes_read: read as u64,
            data: buffer,
        })
    })
}

#[op2(fast)]
#[smi]
pub(crate) fn op_fs_write(
    #[bigint] id: u64,
    #[buffer] data: &[u8],
    #[smi] position: i64,
) -> Result<i32, CoreError> {
    file_with_mut(id, |file| {
        if position >= 0 {
            file.seek(SeekFrom::Start(position as u64)).map_err(|err| {
                CoreError::from(std::io::Error::new(
                    err.kind(),
                    format!("Failed to seek: {}", err),
                ))
            })?;
        }
        let written = file.write(data).map_err(|err| {
            CoreError::from(std::io::Error::new(
                err.kind(),
                format!("Failed to write file: {}", err),
            ))
        })?;
        Ok(written as i32)
    })
}

#[op2(fast)]
pub(crate) fn op_write_file(
    #[string] path: String,
    #[string] contents: String,
) -> Result<(), CoreError> {
    let path = resolve_path(&path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory {}: {}", parent.display(), err),
            ))
        })?;
    }
    std::fs::write(&path, contents).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write {}: {}", path.display(), err),
        ))
    })?;
    Ok(())
}

#[op2(fast)]
pub(crate) fn op_write_file_base64(
    #[string] path: String,
    #[string] contents_base64: String,
) -> Result<(), CoreError> {
    let path = resolve_path(&path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory {}: {}", parent.display(), err),
            ))
        })?;
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(contents_base64.as_bytes())
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to decode base64: {}", err),
            ))
        })?;
    std::fs::write(&path, bytes).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write {}: {}", path.display(), err),
        ))
    })?;
    Ok(())
}
