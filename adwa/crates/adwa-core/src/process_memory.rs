use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};

use crate::{
    Command, ErrorCode, ExitStatus, FileSystem, FileType, MkdirOptions, ProcessHandle, ProcessHost,
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
        let cwd = options
            .cwd
            .as_deref()
            .unwrap_or_else(|| Path::new("/home/user"));
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
            b"commands: help, pwd, ls [path], cat <file>, echo [text], mkdir <dir>, touch <file>, cp <src> <dst>, mv <src> <dst>, rm <path>, deka run <file>, deka db <generate|migrate|info|flush>\n".to_vec(),
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
    if args[0] == "db" {
        return run_deka_db(cwd, &args[1..], fs);
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

fn run_deka_db(cwd: &Path, args: &[String], fs: Option<&dyn FileSystem>) -> Result<CommandOutput> {
    let Some(fs) = fs else {
        return Ok(CommandOutput::fail(
            1,
            b"deka db unavailable: filesystem is not mounted\n".to_vec(),
        ));
    };

    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "generate" => run_deka_db_generate(cwd, &args[1..], fs),
        "info" | "status" => run_deka_db_info(cwd, fs),
        "migrate" => run_deka_db_migrate(cwd, fs),
        "flush" => run_deka_db_flush(cwd, fs),
        _ => Ok(CommandOutput::fail(
            2,
            b"deka db: use generate|migrate|info|flush\n".to_vec(),
        )),
    }
}

fn run_deka_db_generate(cwd: &Path, args: &[String], fs: &dyn FileSystem) -> Result<CommandOutput> {
    let raw_input = args
        .first()
        .map(String::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("types/index.phpx");
    let candidate = resolve_path(cwd, raw_input);
    let source = if let Ok(meta) = fs.stat(&candidate) {
        match meta.file_type {
            FileType::File => candidate,
            FileType::Directory => candidate.join("index.phpx"),
            _ => candidate,
        }
    } else {
        candidate
    };

    let source_bytes = match fs.read_file(&source) {
        Ok(value) => value,
        Err(_) => {
            return Ok(CommandOutput::fail(
                1,
                format!("[db generate] model input not found: {}\n", source.display()).into_bytes(),
            ));
        }
    };
    let source_text = String::from_utf8_lossy(&source_bytes);
    let models = extract_struct_names(&source_text);
    if models.is_empty() {
        return Ok(CommandOutput::fail(
            1,
            format!(
                "[db generate] no struct models found in {}\n",
                source.display()
            )
            .into_bytes(),
        ));
    }

    let (engine, location) = read_db_config(cwd, fs);

    let db_dir = resolve_path(cwd, "db");
    let generated_dir = db_dir.join(".generated");
    let migrations_dir = db_dir.join("migrations");
    let applied_dir = db_dir.join(".applied");
    fs.mkdir(&db_dir, MkdirOptions { recursive: true, mode: None })?;
    fs.mkdir(&generated_dir, MkdirOptions { recursive: true, mode: None })?;
    fs.mkdir(&migrations_dir, MkdirOptions { recursive: true, mode: None })?;
    fs.mkdir(&applied_dir, MkdirOptions { recursive: true, mode: None })?;

    let generated_header = "/*\n * AUTO-GENERATED FILE - DO NOT EDIT\n * Generated by deka db generate\n */\n\n";
    let index_body = format!(
        "{}import {{ connect, query, exec, close, stats }} from 'db'\nexport {{ connect, query, exec, close, stats }}\n",
        generated_header
    );
    let client_body = format!(
        "{}import {{ openHandle as __open, query as __query, exec as __exec, close as __close, stats as __stats }} from 'db'\n\nexport function connect($name = '{}') {{\n  return __open('sqlite', {{ path: $name }})\n}}\n\nexport function query($h, $sql, $params = []) {{\n  return __query($h, $sql, $params)\n}}\n\nexport function exec($h, $sql, $params = []) {{\n  return __exec($h, $sql, $params)\n}}\n\nexport function close($h) {{\n  return __close($h)\n}}\n\nexport function stats() {{\n  return __stats()\n}}\n",
        generated_header,
        json_escape(&location)
    );

    let mut meta_entries = String::new();
    for model in &models {
        meta_entries.push_str(&format!("  '{}' => ['name' => '{}'],\n", model, model));
    }
    let meta_body = format!(
        "{}export function models() {{\n  return [\n{}  ]\n}}\n",
        generated_header, meta_entries
    );

    let migration_body = render_init_migration(&models);
    let schema_body = format!(
        "{{\n  \"models\": [{}],\n  \"engine\": \"{}\",\n  \"location\": \"{}\"\n}}\n",
        models
            .iter()
            .map(|v| format!("\"{}\"", json_escape(v)))
            .collect::<Vec<_>>()
            .join(", "),
        json_escape(&engine),
        json_escape(&location)
    );

    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let state_body = format!(
        "{{\n  \"source\": \"{}\",\n  \"generated_at_unix\": {},\n  \"model_count\": {},\n  \"models\": [{}],\n  \"engine\": \"{}\",\n  \"location\": \"{}\",\n  \"migration_files\": [\"0001_init.sql\"],\n  \"applied_migrations\": [],\n  \"mode\": \"browser\"\n}}\n",
        json_escape(&source.display().to_string()),
        generated_at,
        models.len(),
        models
            .iter()
            .map(|v| format!("\"{}\"", json_escape(v)))
            .collect::<Vec<_>>()
            .join(", "),
        json_escape(&engine),
        json_escape(&location)
    );

    fs.write_file(&db_dir.join("index.phpx"), index_body.as_bytes(), WriteOptions::default())?;
    fs.write_file(&db_dir.join("client.phpx"), client_body.as_bytes(), WriteOptions::default())?;
    fs.write_file(&db_dir.join("meta.phpx"), meta_body.as_bytes(), WriteOptions::default())?;
    fs.write_file(&db_dir.join("_state.json"), state_body.as_bytes(), WriteOptions::default())?;
    fs.write_file(&migrations_dir.join("0001_init.sql"), migration_body.as_bytes(), WriteOptions::default())?;
    fs.write_file(&generated_dir.join("schema.json"), schema_body.as_bytes(), WriteOptions::default())?;

    Ok(CommandOutput::ok(
        format!(
            "[db generate] generated 6 files from {} model(s) in {}\n",
            models.len(),
            source.display()
        )
        .into_bytes(),
    ))
}

fn run_deka_db_info(cwd: &Path, fs: &dyn FileSystem) -> Result<CommandOutput> {
    let state_path = resolve_path(cwd, "db/_state.json");
    let raw = match fs.read_file(&state_path) {
        Ok(v) => String::from_utf8_lossy(&v).to_string(),
        Err(_) => {
            return Ok(CommandOutput::fail(
                1,
                b"[db info] db/_state.json not found. run `deka db generate` first\n".to_vec(),
            ));
        }
    };
    let source = json_extract_string(&raw, "source").unwrap_or_else(|| "unknown".to_string());
    let models = json_extract_number(&raw, "model_count").unwrap_or(0);
    let generated = json_extract_number(&raw, "generated_at_unix").unwrap_or(0);
    let engine = json_extract_string(&raw, "engine").unwrap_or_else(|| "sqlite".to_string());
    let location = json_extract_string(&raw, "location").unwrap_or_else(|| "./deka.sqlite".to_string());

    let migration_count = count_sql_files(&resolve_path(cwd, "db/migrations"), fs);
    let applied_count = count_applied_markers(&resolve_path(cwd, "db/.applied"), fs);
    let pending = migration_count.saturating_sub(applied_count);

    let out = format!(
        "[db info] source: {}\n[db info] models: {}\n[db info] generated_at_unix: {}\n[db info] engine: {}\n[db info] location: {}\n[db info] migration_files: {}\n[db info] applied_migrations: {}\n[db info] pending_migrations: {}\n[db info] mode: browser\n",
        source, models, generated, engine, location, migration_count, applied_count, pending
    );
    Ok(CommandOutput::ok(out.into_bytes()))
}

fn run_deka_db_migrate(cwd: &Path, fs: &dyn FileSystem) -> Result<CommandOutput> {
    let migrations_dir = resolve_path(cwd, "db/migrations");
    let applied_dir = resolve_path(cwd, "db/.applied");
    fs.mkdir(&applied_dir, MkdirOptions { recursive: true, mode: None })?;

    let mut migrations = list_sql_migrations(&migrations_dir, fs);
    migrations.sort();
    if migrations.is_empty() {
        return Ok(CommandOutput::fail(
            1,
            b"[db migrate] db/migrations directory not found. run `deka db generate` first\n".to_vec(),
        ));
    }

    let applied = list_applied_markers(&applied_dir, fs);
    let mut newly = Vec::new();
    for name in migrations {
        if applied.contains(&name) {
            continue;
        }
        let marker = applied_dir.join(format!("{}.applied", name));
        fs.write_file(&marker, b"ok\n", WriteOptions::default())?;
        newly.push(name);
    }

    if newly.is_empty() {
        return Ok(CommandOutput::ok(b"[db migrate] no pending migrations\n".to_vec()));
    }

    let out = format!(
        "[db migrate] applied {} migration(s): {}\n[db migrate] note: browser mode currently updates migration state metadata only\n",
        newly.len(),
        newly.join(", ")
    );
    Ok(CommandOutput::ok(out.into_bytes()))
}

fn run_deka_db_flush(cwd: &Path, fs: &dyn FileSystem) -> Result<CommandOutput> {
    let applied_dir = resolve_path(cwd, "db/.applied");
    let _ = fs.remove(
        &applied_dir,
        RemoveOptions {
            recursive: true,
            force: true,
        },
    );
    fs.mkdir(&applied_dir, MkdirOptions { recursive: true, mode: None })?;
    Ok(CommandOutput::ok(
        b"[db flush] cleared applied migration metadata in browser mode\n".to_vec(),
    ))
}

fn read_db_config(cwd: &Path, fs: &dyn FileSystem) -> (String, String) {
    let path = resolve_path(cwd, "deka.json");
    let raw = fs
        .read_file(&path)
        .ok()
        .map(|v| String::from_utf8_lossy(&v).to_string())
        .unwrap_or_default();
    let engine = json_extract_string(&raw, "engine").unwrap_or_else(|| "sqlite".to_string());
    let location = json_extract_string(&raw, "location").unwrap_or_else(|| "./deka.sqlite".to_string());
    (engine, location)
}

fn extract_struct_names(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::<String>::new();
    while i + 6 < bytes.len() {
        if source[i..].starts_with("struct") {
            let before_ok = i == 0 || !is_ident_char(bytes[i - 1]);
            let after = i + 6;
            let after_ok = after >= bytes.len() || !is_ident_char(bytes[after]);
            if before_ok && after_ok {
                let mut j = after;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let start = j;
                while j < bytes.len() && is_ident_char(bytes[j]) {
                    j += 1;
                }
                if j > start {
                    let name = &source[start..j];
                    let first = name.as_bytes().first().copied().unwrap_or_default();
                    if first.is_ascii_alphabetic() || first == b'_' {
                        out.push(name.to_string());
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

fn is_ident_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'$'
}

fn render_init_migration(models: &[String]) -> String {
    let mut out = String::from("-- Generated by deka db generate (browser mode)\n\n");
    for model in models {
        let table = to_table_name(model);
        out.push_str(&format!(
            "create table if not exists \"{}\" (\n  id integer primary key,\n  name text not null,\n  version text\n);\n\n",
            table
        ));
    }
    out
}

fn to_table_name(model: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in model.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch.to_ascii_lowercase());
        }
    }
    if !out.ends_with('s') {
        out.push('s');
    }
    out
}

fn json_escape(value: &str) -> String {
    value
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
}

fn json_extract_string(raw: &str, key: &str) -> Option<String> {




    let needle = format!("\"{}\"", key);
    let pos = raw.find(&needle)?;
    let rest = &raw[pos + needle.len()..];
    let colon = rest.find(':')?;
    let mut val = rest[colon + 1..].trim_start();
    if !val.starts_with('"') {
        return None;
    }
    val = &val[1..];
    let end = val.find('"')?;
    Some(val[..end].to_string())
}

fn json_extract_number(raw: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{}\"", key);
    let pos = raw.find(&needle)?;
    let rest = &raw[pos + needle.len()..];
    let colon = rest.find(':')?;
    let val = rest[colon + 1..].trim_start();
    let end = val.find(|c: char| !c.is_ascii_digit()).unwrap_or(val.len());
    val[..end].parse::<usize>().ok()
}

fn list_sql_migrations(dir: &Path, fs: &dyn FileSystem) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = fs.readdir(dir) {
        for entry in entries {
            if entry.file_type != FileType::File {
                continue;
            }
            if entry.name.ends_with(".sql") {
                out.push(entry.name);
            }
        }
    }
    out
}

fn count_sql_files(dir: &Path, fs: &dyn FileSystem) -> usize {
    list_sql_migrations(dir, fs).len()
}

fn list_applied_markers(dir: &Path, fs: &dyn FileSystem) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Ok(entries) = fs.readdir(dir) {
        for entry in entries {
            if entry.file_type != FileType::File {
                continue;
            }
            if let Some(name) = entry.name.strip_suffix(".applied") {
                out.insert(name.to_string());
            }
        }
    }
    out
}

fn count_applied_markers(dir: &Path, fs: &dyn FileSystem) -> usize {
    list_applied_markers(dir, fs).len()
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
                    cwd: Some(std::path::PathBuf::from("/")),
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
    #[test]
    fn deka_db_generate_and_lifecycle() {
        let fs = Arc::new(InMemoryFileSystem::new());
        fs.mkdir(
            std::path::Path::new("/types"),
            crate::MkdirOptions {
                recursive: true,
                mode: None,
            },
        )
        .expect("mkdir /types");
        fs.write_file(
            std::path::Path::new("/types/index.phpx"),
            b"struct Package {\n  $name: string\n  $version: string\n}\n",
            WriteOptions {
                create: true,
                truncate: true,
                mode: None,
            },
        )
        .expect("write model file");

        let host = InMemoryProcessHost::with_fs(fs.clone());

        let run_cmd = |args: Vec<&str>| -> (i32, String, String) {
            let mut handle = host
                .spawn(
                    Command {
                        program: "deka".to_string(),
                        args: args.into_iter().map(|v| v.to_string()).collect(),
                    },
                    SpawnOptions {
                        stdout: StdioMode::Piped,
                        stderr: StdioMode::Piped,
                        cwd: Some(std::path::PathBuf::from("/")),
                        ..SpawnOptions::default()
                    },
                )
                .expect("spawn deka db command");

            let mut out = vec![0u8; 8192];
            let out_len = handle
                .stdout()
                .expect("stdout stream")
                .read(&mut out)
                .expect("read stdout");
            let stdout = String::from_utf8_lossy(&out[..out_len]).to_string();

            let mut err = vec![0u8; 4096];
            let err_len = handle
                .stderr()
                .expect("stderr stream")
                .read(&mut err)
                .expect("read stderr");
            let stderr = String::from_utf8_lossy(&err[..err_len]).to_string();

            let status = handle.wait().expect("wait command");
            (status.code, stdout, stderr)
        };

        let (code_gen, out_gen, err_gen) = run_cmd(vec!["db", "generate"]);
        assert_eq!(code_gen, 0, "generate stderr={}", err_gen);
        assert!(out_gen.contains("[db generate]"), "stdout={}", out_gen);
        assert!(
            fs.read_file(std::path::Path::new("/db/_state.json")).is_ok(),
            "expected /db/_state.json"
        );

        let (code_info, out_info, err_info) = run_cmd(vec!["db", "info"]);
        assert_eq!(code_info, 0, "info stderr={}", err_info);
        assert!(out_info.contains("[db info] engine:"), "stdout={}", out_info);

        let (code_mig, out_mig, err_mig) = run_cmd(vec!["db", "migrate"]);
        assert_eq!(code_mig, 0, "migrate stderr={}", err_mig);
        assert!(
            out_mig.contains("[db migrate] applied") || out_mig.contains("[db migrate] no pending migrations"),
            "stdout={}",
            out_mig
        );

        let (code_flush, out_flush, err_flush) = run_cmd(vec!["db", "flush"]);
        assert_eq!(code_flush, 0, "flush stderr={}", err_flush);
        assert!(out_flush.contains("[db flush]"), "stdout={}", out_flush);
    }

}
