use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use js_sys::{Array, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use wosix_core::{
    Command, ExitStatus, FileSystem, FileType, FsEvent, FsEventKind, FsWatcher, InMemoryFileSystem,
    InMemoryNetHost, InMemoryProcessHost, MkdirOptions, MountFile, MountTree, NetHost, PortEvent,
    PortInfo, PortProtocol, PortPublishOptions, ProcessHandle as CoreProcessHandle, ProcessHost,
    ProcessId, ProcessSignal, RemoveOptions, SpawnOptions, StdioMode, WatchOptions, WosixError,
    WriteOptions,
};

#[wasm_bindgen]
pub struct WebContainer {
    fs: Arc<InMemoryFileSystem>,
    process: Arc<InMemoryProcessHost>,
    net: Arc<InMemoryNetHost>,
    processes: RefCell<BTreeMap<u32, Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>>>,
    process_meta: RefCell<BTreeMap<u32, ProcessMeta>>,
    foreground_pid: Cell<u32>,
    port_listeners: RefCell<BTreeMap<u32, js_sys::Function>>,
    next_port_listener: Cell<u32>,
}

#[derive(Clone, Debug)]
struct ProcessMeta {
    program: String,
    args: Vec<String>,
    cwd: String,
    ports: Vec<u16>,
}

#[wasm_bindgen]
impl WebContainer {
    #[wasm_bindgen(js_name = boot)]
    pub fn boot() -> WebContainer {
        let fs = Arc::new(InMemoryFileSystem::new());
        WebContainer {
            fs: fs.clone(),
            process: Arc::new(InMemoryProcessHost::with_fs(fs)),
            net: Arc::new(InMemoryNetHost::new()),
            processes: RefCell::new(BTreeMap::new()),
            process_meta: RefCell::new(BTreeMap::new()),
            foreground_pid: Cell::new(0),
            port_listeners: RefCell::new(BTreeMap::new()),
            next_port_listener: Cell::new(1),
        }
    }

    #[wasm_bindgen(js_name = fs)]
    pub fn fs(&self) -> FsHandle {
        FsHandle {
            fs: self.fs.clone(),
        }
    }

    #[wasm_bindgen(js_name = spawn)]
    pub fn spawn(
        &self,
        program: &str,
        args: js_sys::Array,
        options: Option<JsValue>,
    ) -> Result<ProcessHandle, JsValue> {
        let args = array_to_strings(&args)?;
        let mut spawn_options = SpawnOptions::default();
        spawn_options.cwd = options_string(&options, "cwd")?.map(Into::into);
        spawn_options.clear_env = options_bool(&options, "clearEnv", false)?;
        spawn_options.pty = options_bool(&options, "pty", false)?;
        spawn_options.env = options_string_map(&options, "env")?;
        spawn_options.stdin = options_stdio(&options, "stdin", StdioMode::Piped)?;
        spawn_options.stdout = options_stdio(&options, "stdout", StdioMode::Piped)?;
        spawn_options.stderr = options_stdio(&options, "stderr", StdioMode::Piped)?;
        let command = Command {
            program: program.to_string(),
            args,
        };
        let program_name = command.program.clone();
        let command_args = command.args.clone();
        let published_ports = ports_for_spawn_command(&command);
        let handle = self
            .process
            .spawn(command, spawn_options)
            .map_err(err_to_js)?;
        let wrapped = ProcessHandle::new(handle);
        let pid = wrapped.pid();
        self.processes.borrow_mut().insert(pid, wrapped.handle_ref());
        self.process_meta.borrow_mut().insert(
            pid,
            ProcessMeta {
                program: program_name,
                args: command_args,
                cwd: options_string(&options, "cwd")?.unwrap_or_else(|| "/".to_string()),
                ports: published_ports.clone(),
            },
        );
        for port in published_ports {
            let info = self
                .net
                .publish_port(
                    port,
                    PortPublishOptions {
                        protocol: PortProtocol::Http,
                        host: Some("localhost".to_string()),
                    },
                )
                .map_err(err_to_js)?;
            let _ = self.dispatch_port_event(PortEvent::ServerReady(info));
        }
        Ok(wrapped)
    }

    #[wasm_bindgen(js_name = setForegroundPid)]
    pub fn set_foreground_pid(&self, pid: u32) -> Result<(), JsValue> {
        if pid == 0 {
            self.foreground_pid.set(0);
            return Ok(());
        }
        if !self.processes.borrow().contains_key(&pid) {
            return Err(js_error("unknown process id"));
        }
        self.foreground_pid.set(pid);
        Ok(())
    }

    #[wasm_bindgen(js_name = clearForegroundPid)]
    pub fn clear_foreground_pid(&self) {
        self.foreground_pid.set(0);
    }

    #[wasm_bindgen(js_name = foregroundPid)]
    pub fn foreground_pid(&self) -> JsValue {
        let pid = self.foreground_pid.get();
        if pid == 0 {
            JsValue::NULL
        } else {
            JsValue::from_f64(pid as f64)
        }
    }

    #[wasm_bindgen(js_name = signalForeground)]
    pub fn signal_foreground(&self, signal: Option<i32>) -> Result<bool, JsValue> {
        let pid = self.foreground_pid.get();
        if pid == 0 {
            return Ok(false);
        }
        let Some(handle) = self.processes.borrow().get(&pid).cloned() else {
            self.foreground_pid.set(0);
            return Ok(false);
        };
        if let Some(proc) = handle.borrow_mut().as_mut() {
            proc.kill(parse_signal(signal)).map_err(err_to_js)?;
            self.cleanup_process_ports(pid)?;
            self.foreground_pid.set(0);
            return Ok(true);
        }
        self.cleanup_process_ports(pid)?;
        self.foreground_pid.set(0);
        Ok(false)
    }

    #[wasm_bindgen(js_name = listProcesses)]
    pub fn list_processes(&self) -> Result<JsValue, JsValue> {
        let array = Array::new();
        let process_meta = self.process_meta.borrow();
        let processes = self.processes.borrow();

        for (pid, meta) in process_meta.iter() {
            let obj = Object::new();
            Reflect::set(
                &obj,
                &JsValue::from_str("pid"),
                &JsValue::from_f64(*pid as f64),
            )?;
            Reflect::set(
                &obj,
                &JsValue::from_str("program"),
                &JsValue::from_str(&meta.program),
            )?;
            let args = Array::new();
            for arg in &meta.args {
                args.push(&JsValue::from_str(arg));
            }
            Reflect::set(&obj, &JsValue::from_str("args"), &args)?;
            Reflect::set(
                &obj,
                &JsValue::from_str("cwd"),
                &JsValue::from_str(&meta.cwd),
            )?;
            let ports = Array::new();
            for port in &meta.ports {
                ports.push(&JsValue::from_f64(*port as f64));
            }
            Reflect::set(&obj, &JsValue::from_str("ports"), &ports)?;

            let running = if let Some(handle) = processes.get(pid) {
                handle.borrow().is_some()
            } else {
                false
            };
            Reflect::set(&obj, &JsValue::from_str("running"), &JsValue::from_bool(running))?;
            array.push(&obj);
        }

        Ok(array.into())
    }

    #[wasm_bindgen(js_name = publishPort)]
    pub fn publish_port(&self, port: u16, options: Option<JsValue>) -> Result<JsValue, JsValue> {
        let protocol = options_protocol(&options, "protocol", PortProtocol::Http)?;
        let host = options_string(&options, "host")?;
        let info = self
            .net
            .publish_port(port, PortPublishOptions { protocol, host })
            .map_err(err_to_js)?;
        let _ = self.dispatch_port_event(PortEvent::ServerReady(info.clone()));
        port_info_to_js(info)
    }

    #[wasm_bindgen(js_name = unpublishPort)]
    pub fn unpublish_port(&self, port: u16) -> Result<(), JsValue> {
        self.net.unpublish_port(port).map_err(err_to_js)?;
        let _ = self.dispatch_port_event(PortEvent::PortClosed(port));
        Ok(())
    }

    #[wasm_bindgen(js_name = nextPortEvent)]
    pub fn next_port_event(&self) -> Result<JsValue, JsValue> {
        let event = self.net.next_event().map_err(err_to_js)?;
        match event {
            Some(event) => port_event_to_js(event),
            None => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = onPortEvent)]
    pub fn on_port_event(&self, callback: js_sys::Function) -> u32 {
        let id = self.next_port_listener.get();
        self.next_port_listener.set(id + 1);
        self.port_listeners.borrow_mut().insert(id, callback);
        id
    }

    #[wasm_bindgen(js_name = offPortEvent)]
    pub fn off_port_event(&self, id: u32) {
        self.port_listeners.borrow_mut().remove(&id);
    }

    fn dispatch_port_event(&self, event: PortEvent) -> Result<(), JsValue> {
        let payload = port_event_to_js(event)?;
        for listener in self.port_listeners.borrow().values() {
            let _ = listener.call1(&JsValue::NULL, &payload);
        }
        Ok(())
    }

    fn cleanup_process_ports(&self, pid: u32) -> Result<(), JsValue> {
        let ports = self
            .process_meta
            .borrow_mut()
            .remove(&pid)
            .map(|meta| meta.ports)
            .unwrap_or_default();
        self.processes.borrow_mut().remove(&pid);
        for port in ports {
            self.net.unpublish_port(port).map_err(err_to_js)?;
            let _ = self.dispatch_port_event(PortEvent::PortClosed(port));
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub struct FsHandle {
    fs: Arc<InMemoryFileSystem>,
}

#[wasm_bindgen]
impl FsHandle {
    #[wasm_bindgen(js_name = readFile)]
    pub fn read_file(&self, path: &str) -> Result<Uint8Array, JsValue> {
        let data = self.fs.read_file(Path::new(path)).map_err(err_to_js)?;
        Ok(Uint8Array::from(data.as_slice()))
    }

    #[wasm_bindgen(js_name = writeFile)]
    pub fn write_file(
        &self,
        path: &str,
        data: Uint8Array,
        options: Option<JsValue>,
    ) -> Result<(), JsValue> {
        let create = options_bool(&options, "create", true)?;
        let truncate = options_bool(&options, "truncate", true)?;
        let opts = WriteOptions {
            create,
            truncate,
            mode: None,
        };
        self.fs
            .write_file(Path::new(path), &data.to_vec(), opts)
            .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = readdir)]
    pub fn readdir(&self, path: &str) -> Result<js_sys::Array, JsValue> {
        let entries = self.fs.readdir(Path::new(path)).map_err(err_to_js)?;
        let array = js_sys::Array::new();
        for entry in entries {
            array.push(&JsValue::from_str(&entry.name));
        }
        Ok(array)
    }

    #[wasm_bindgen(js_name = mkdir)]
    pub fn mkdir(&self, path: &str, options: Option<JsValue>) -> Result<(), JsValue> {
        let recursive = options_bool(&options, "recursive", false)?;
        let opts = MkdirOptions {
            recursive,
            mode: None,
        };
        self.fs.mkdir(Path::new(path), opts).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = rm)]
    pub fn rm(&self, path: &str, options: Option<JsValue>) -> Result<(), JsValue> {
        let recursive = options_bool(&options, "recursive", false)?;
        let force = options_bool(&options, "force", false)?;
        let opts = RemoveOptions { recursive, force };
        self.fs.remove(Path::new(path), opts).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = rename)]
    pub fn rename(&self, from: &str, to: &str) -> Result<(), JsValue> {
        self.fs
            .rename(Path::new(from), Path::new(to))
            .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = stat)]
    pub fn stat(&self, path: &str) -> Result<JsValue, JsValue> {
        let meta = self.fs.stat(Path::new(path)).map_err(err_to_js)?;
        let obj = Object::new();
        Reflect::set(
            &obj,
            &JsValue::from_str("size"),
            &JsValue::from_f64(meta.size as f64),
        )?;
        Reflect::set(
            &obj,
            &JsValue::from_str("fileType"),
            &JsValue::from_str(file_type_label(meta.file_type)),
        )?;
        Ok(obj.into())
    }

    #[wasm_bindgen(js_name = mount)]
    pub fn mount(&self, tree: JsValue) -> Result<(), JsValue> {
        let mount_tree = js_to_mount_tree(tree)?;
        self.fs
            .mount_tree(Path::new("/"), mount_tree)
            .map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = watch)]
    pub fn watch(&self, path: &str, options: Option<JsValue>) -> Result<FsWatchHandle, JsValue> {
        let recursive = options_bool(&options, "recursive", true)?;
        let opts = WatchOptions { recursive };
        let watcher = self.fs.watch(Path::new(path), opts).map_err(err_to_js)?;
        Ok(FsWatchHandle::new(watcher))
    }
}

#[wasm_bindgen]
pub struct FsWatchHandle {
    watcher: RefCell<Option<Box<dyn FsWatcher>>>,
}

#[wasm_bindgen]
pub struct ProcessHandle {
    handle: Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>,
    id: ProcessId,
    readable_closures: RefCell<Vec<Closure<dyn FnMut(JsValue) -> JsValue>>>,
    writable_closures: RefCell<Vec<Closure<dyn FnMut(JsValue, JsValue) -> JsValue>>>,
}

#[wasm_bindgen]
impl ProcessHandle {
    fn new(handle: Box<dyn CoreProcessHandle>) -> ProcessHandle {
        let id = handle.id();
        ProcessHandle {
            handle: Rc::new(RefCell::new(Some(handle))),
            id,
            readable_closures: RefCell::new(Vec::new()),
            writable_closures: RefCell::new(Vec::new()),
        }
    }

    fn handle_ref(&self) -> Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>> {
        self.handle.clone()
    }

    #[wasm_bindgen(js_name = pid)]
    pub fn pid(&self) -> u32 {
        self.id.0
    }

    #[wasm_bindgen(js_name = wait)]
    pub fn wait(&self) -> Result<JsValue, JsValue> {
        let mut handle = self.handle.borrow_mut();
        let handle = handle
            .as_mut()
            .ok_or_else(|| js_error("process handle is closed"))?;
        let status = handle.wait().map_err(err_to_js)?;
        exit_status_to_js(status)
    }

    #[wasm_bindgen(js_name = exit)]
    pub fn exit(&self) -> Promise {
        match self.wait() {
            Ok(value) => Promise::resolve(&value),
            Err(err) => Promise::reject(&err),
        }
    }

    #[wasm_bindgen(js_name = writeStdin)]
    pub fn write_stdin(&self, data: JsValue) -> Result<u32, JsValue> {
        let bytes = stdin_bytes(data)?;
        let written = write_to_stdin(&self.handle, &bytes)?;
        Ok(written as u32)
    }

    #[wasm_bindgen(js_name = readStdout)]
    pub fn read_stdout(&self, max_bytes: Option<u32>) -> Result<JsValue, JsValue> {
        read_stream(self, StreamKind::Stdout, max_bytes)
    }

    #[wasm_bindgen(js_name = readStderr)]
    pub fn read_stderr(&self, max_bytes: Option<u32>) -> Result<JsValue, JsValue> {
        read_stream(self, StreamKind::Stderr, max_bytes)
    }

    #[wasm_bindgen(js_name = readOutput)]
    pub fn read_output(&self, max_bytes: Option<u32>) -> Result<JsValue, JsValue> {
        read_combined_stream(self, max_bytes)
    }

    #[wasm_bindgen(js_name = stdinStream)]
    pub fn stdin_stream(&self) -> Result<JsValue, JsValue> {
        let handle = self.handle.clone();
        let write_closure = Closure::wrap(Box::new(move |chunk: JsValue, _controller: JsValue| {
            let result = stdin_bytes(chunk).and_then(|bytes| write_to_stdin(&handle, &bytes));
            match result {
                Ok(_) => JsValue::UNDEFINED,
                Err(err) => Promise::reject(&err).into(),
            }
        })
            as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
        let sink = Object::new();
        Reflect::set(&sink, &JsValue::from_str("write"), write_closure.as_ref())?;
        self.writable_closures.borrow_mut().push(write_closure);
        construct_stream("WritableStream", &sink)
    }

    #[wasm_bindgen(js_name = stdoutStream)]
    pub fn stdout_stream(&self) -> Result<JsValue, JsValue> {
        create_readable_stream(&self.handle, StreamKind::Stdout, &self.readable_closures)
    }

    #[wasm_bindgen(js_name = stderrStream)]
    pub fn stderr_stream(&self) -> Result<JsValue, JsValue> {
        create_readable_stream(&self.handle, StreamKind::Stderr, &self.readable_closures)
    }

    #[wasm_bindgen(js_name = outputStream)]
    pub fn output_stream(&self) -> Result<JsValue, JsValue> {
        create_output_stream(&self.handle, &self.readable_closures)
    }

    #[wasm_bindgen(js_name = kill)]
    pub fn kill(&self, signal: Option<i32>) -> Result<(), JsValue> {
        let mut handle = self.handle.borrow_mut();
        let handle = handle
            .as_mut()
            .ok_or_else(|| js_error("process handle is closed"))?;
        handle.kill(parse_signal(signal)).map_err(err_to_js)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = close)]
    pub fn close(&self) {
        let mut handle = self.handle.borrow_mut();
        handle.take();
        self.readable_closures.borrow_mut().clear();
        self.writable_closures.borrow_mut().clear();
    }
}

#[wasm_bindgen]
impl FsWatchHandle {
    fn new(watcher: Box<dyn FsWatcher>) -> FsWatchHandle {
        FsWatchHandle {
            watcher: RefCell::new(Some(watcher)),
        }
    }

    #[wasm_bindgen(js_name = nextEvent)]
    pub fn next_event(&self) -> Result<JsValue, JsValue> {
        let mut watcher = self.watcher.borrow_mut();
        let watcher = watcher
            .as_mut()
            .ok_or_else(|| js_error("watcher is closed"))?;
        let event = watcher.next_event().map_err(err_to_js)?;
        match event {
            Some(event) => Ok(event_to_js(event)?),
            None => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = close)]
    pub fn close(&self) -> Result<(), JsValue> {
        let mut watcher = self.watcher.borrow_mut();
        if let Some(mut watcher) = watcher.take() {
            watcher.close().map_err(err_to_js)?;
        }
        Ok(())
    }
}

fn event_to_js(event: FsEvent) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    Reflect::set(
        &obj,
        &JsValue::from_str("path"),
        &JsValue::from_str(&event.path),
    )?;
    Reflect::set(
        &obj,
        &JsValue::from_str("kind"),
        &JsValue::from_str(event_kind_label(event.kind)),
    )?;
    if let Some(target) = event.target_path {
        Reflect::set(
            &obj,
            &JsValue::from_str("targetPath"),
            &JsValue::from_str(&target),
        )?;
    }
    Ok(obj.into())
}

fn exit_status_to_js(status: ExitStatus) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    Reflect::set(
        &obj,
        &JsValue::from_str("code"),
        &JsValue::from_f64(status.code as f64),
    )?;
    let signal = status.signal.map(signal_to_int);
    if let Some(signal) = signal {
        Reflect::set(
            &obj,
            &JsValue::from_str("signal"),
            &JsValue::from_f64(signal as f64),
        )?;
    }
    Ok(obj.into())
}

#[derive(Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}

fn read_stream(
    handle: &ProcessHandle,
    kind: StreamKind,
    max_bytes: Option<u32>,
) -> Result<JsValue, JsValue> {
    let max_bytes = max_bytes.unwrap_or(65_536) as usize;
    match read_stream_inner(&handle.handle, kind, max_bytes)? {
        Some(bytes) => Ok(Uint8Array::from(bytes.as_slice()).into()),
        None => Ok(JsValue::NULL),
    }
}

fn read_combined_stream(
    handle: &ProcessHandle,
    max_bytes: Option<u32>,
) -> Result<JsValue, JsValue> {
    let max_bytes = max_bytes.unwrap_or(65_536) as usize;
    match read_combined_inner(&handle.handle, max_bytes)? {
        Some(bytes) => Ok(Uint8Array::from(bytes.as_slice()).into()),
        None => Ok(JsValue::NULL),
    }
}

fn read_stream_inner(
    handle: &Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>,
    kind: StreamKind,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, JsValue> {
    let mut handle = handle.borrow_mut();
    let handle = handle
        .as_mut()
        .ok_or_else(|| js_error("process handle is closed"))?;
    let stream = match kind {
        StreamKind::Stdout => handle.stdout(),
        StreamKind::Stderr => handle.stderr(),
    };
    let Some(stream) = stream else {
        return Ok(None);
    };
    let mut buffer = vec![0u8; max_bytes];
    let read = stream.read(&mut buffer).map_err(err_to_js)?;
    if read == 0 {
        return Ok(None);
    }
    buffer.truncate(read);
    Ok(Some(buffer))
}

fn read_combined_inner(
    handle: &Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, JsValue> {
    if let Some(bytes) = read_stream_inner(handle, StreamKind::Stdout, max_bytes)? {
        return Ok(Some(bytes));
    }
    read_stream_inner(handle, StreamKind::Stderr, max_bytes)
}

fn create_readable_stream(
    handle: &Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>,
    kind: StreamKind,
    store: &RefCell<Vec<Closure<dyn FnMut(JsValue) -> JsValue>>>,
) -> Result<JsValue, JsValue> {
    let handle = handle.clone();
    let pull_closure = Closure::wrap(Box::new(move |controller: JsValue| {
        match read_stream_inner(&handle, kind, 65_536) {
            Ok(Some(bytes)) => {
                let chunk = Uint8Array::from(bytes.as_slice());
                let chunk = JsValue::from(chunk);
                let _ = controller_call(&controller, "enqueue", Some(&chunk));
            }
            Ok(None) => {}
            Err(err) => {
                let _ = controller_call(&controller, "error", Some(&err));
            }
        }
        JsValue::UNDEFINED
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    let source = Object::new();
    Reflect::set(&source, &JsValue::from_str("pull"), pull_closure.as_ref())?;
    store.borrow_mut().push(pull_closure);
    construct_stream("ReadableStream", &source)
}

fn create_output_stream(
    handle: &Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>,
    store: &RefCell<Vec<Closure<dyn FnMut(JsValue) -> JsValue>>>,
) -> Result<JsValue, JsValue> {
    let handle = handle.clone();
    let pull_closure = Closure::wrap(Box::new(move |controller: JsValue| {
        match read_combined_inner(&handle, 65_536) {
            Ok(Some(bytes)) => {
                let chunk = Uint8Array::from(bytes.as_slice());
                let chunk = JsValue::from(chunk);
                let _ = controller_call(&controller, "enqueue", Some(&chunk));
            }
            Ok(None) => {}
            Err(err) => {
                let _ = controller_call(&controller, "error", Some(&err));
            }
        }
        JsValue::UNDEFINED
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    let source = Object::new();
    Reflect::set(&source, &JsValue::from_str("pull"), pull_closure.as_ref())?;
    store.borrow_mut().push(pull_closure);
    construct_stream("ReadableStream", &source)
}

fn construct_stream(name: &str, underlying: &Object) -> Result<JsValue, JsValue> {
    let ctor = Reflect::get(&js_sys::global(), &JsValue::from_str(name))?;
    let ctor: js_sys::Function = ctor
        .dyn_into()
        .map_err(|_| js_error("stream constructor not available"))?;
    let args = Array::new();
    args.push(&underlying.clone().into());
    Reflect::construct(&ctor, &args)
}

fn controller_call(
    controller: &JsValue,
    method: &str,
    arg: Option<&JsValue>,
) -> Result<(), JsValue> {
    let func = Reflect::get(controller, &JsValue::from_str(method))?;
    let func: js_sys::Function = func
        .dyn_into()
        .map_err(|_| js_error("stream controller method is not callable"))?;
    match arg {
        Some(arg) => {
            func.call1(controller, arg)?;
        }
        None => {
            func.call0(controller)?;
        }
    }
    Ok(())
}

fn stdin_bytes(data: JsValue) -> Result<Vec<u8>, JsValue> {
    if data.is_string() {
        Ok(data
            .as_string()
            .ok_or_else(|| js_error("stdin string must be utf-8"))?
            .into_bytes())
    } else if data.is_instance_of::<Uint8Array>() {
        Ok(Uint8Array::new(&data).to_vec())
    } else {
        Err(js_error("stdin data must be a string or Uint8Array"))
    }
}

fn write_to_stdin(
    handle: &Rc<RefCell<Option<Box<dyn CoreProcessHandle>>>>,
    bytes: &[u8],
) -> Result<usize, JsValue> {
    let mut handle = handle.borrow_mut();
    let handle = handle
        .as_mut()
        .ok_or_else(|| js_error("process handle is closed"))?;
    let Some(stdin) = handle.stdin() else {
        return Err(js_error("stdin is not piped"));
    };
    stdin.write(bytes).map_err(err_to_js)
}

fn port_info_to_js(info: PortInfo) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    Reflect::set(
        &obj,
        &JsValue::from_str("port"),
        &JsValue::from_f64(info.port as f64),
    )?;
    Reflect::set(
        &obj,
        &JsValue::from_str("url"),
        &JsValue::from_str(&info.url),
    )?;
    Reflect::set(
        &obj,
        &JsValue::from_str("protocol"),
        &JsValue::from_str(protocol_label(info.protocol)),
    )?;
    Ok(obj.into())
}

fn port_event_to_js(event: PortEvent) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    match event {
        PortEvent::ServerReady(info) => {
            Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("server-ready"),
            )?;
            Reflect::set(
                &obj,
                &JsValue::from_str("port"),
                &JsValue::from_f64(info.port as f64),
            )?;
            Reflect::set(
                &obj,
                &JsValue::from_str("url"),
                &JsValue::from_str(&info.url),
            )?;
            Reflect::set(
                &obj,
                &JsValue::from_str("protocol"),
                &JsValue::from_str(protocol_label(info.protocol)),
            )?;
        }
        PortEvent::PortClosed(port) => {
            Reflect::set(
                &obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str("port-closed"),
            )?;
            Reflect::set(
                &obj,
                &JsValue::from_str("port"),
                &JsValue::from_f64(port as f64),
            )?;
        }
    }
    Ok(obj.into())
}

fn options_bool(options: &Option<JsValue>, key: &str, default: bool) -> Result<bool, JsValue> {
    let Some(options) = options else {
        return Ok(default);
    };
    if options.is_null() || options.is_undefined() {
        return Ok(default);
    }
    if !options.is_object() {
        return Err(js_error("options must be an object"));
    }
    let value = Reflect::get(options, &JsValue::from_str(key))?;
    if value.is_null() || value.is_undefined() {
        return Ok(default);
    }
    Ok(value.as_bool().unwrap_or(default))
}

fn options_string(options: &Option<JsValue>, key: &str) -> Result<Option<String>, JsValue> {
    let Some(options) = options else {
        return Ok(None);
    };
    if options.is_null() || options.is_undefined() {
        return Ok(None);
    }
    if !options.is_object() {
        return Err(js_error("options must be an object"));
    }
    let value = Reflect::get(options, &JsValue::from_str(key))?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .as_string()
        .ok_or_else(|| js_error("option value must be a string"))
        .map(Some)
}

fn options_string_map(
    options: &Option<JsValue>,
    key: &str,
) -> Result<BTreeMap<String, String>, JsValue> {
    let Some(options) = options else {
        return Ok(BTreeMap::new());
    };
    if options.is_null() || options.is_undefined() {
        return Ok(BTreeMap::new());
    }
    if !options.is_object() {
        return Err(js_error("options must be an object"));
    }
    let value = Reflect::get(options, &JsValue::from_str(key))?;
    if value.is_null() || value.is_undefined() {
        return Ok(BTreeMap::new());
    }
    if !value.is_object() {
        return Err(js_error("env must be an object"));
    }
    let obj: Object = value
        .dyn_into()
        .map_err(|_| js_error("env must be an object"))?;
    let mut map = BTreeMap::new();
    for key in Object::keys(&obj).iter() {
        let name = key
            .as_string()
            .ok_or_else(|| js_error("env key must be a string"))?;
        let value = Reflect::get(&obj, &key)?;
        let value = value
            .as_string()
            .ok_or_else(|| js_error("env value must be a string"))?;
        map.insert(name, value);
    }
    Ok(map)
}

fn ports_for_spawn_command(command: &Command) -> Vec<u16> {
    if command.program != "deka" {
        return Vec::new();
    }
    if command.args.first().map(String::as_str) != Some("serve") {
        return Vec::new();
    }
    let mut port = 8530u16;
    let mut i = 1usize;
    while i < command.args.len() {
        if command.args[i] == "--port" {
            if let Some(next) = command.args.get(i + 1) {
                if let Ok(parsed) = next.parse::<u16>() {
                    port = parsed;
                }
                i += 1;
            }
        }
        i += 1;
    }
    vec![port]
}

fn options_stdio(
    options: &Option<JsValue>,
    key: &str,
    default: StdioMode,
) -> Result<StdioMode, JsValue> {
    let value = options_string(options, key)?;
    let Some(value) = value else {
        return Ok(default);
    };
    match value.as_str() {
        "inherit" => Ok(StdioMode::Inherit),
        "piped" | "pipe" => Ok(StdioMode::Piped),
        "null" => Ok(StdioMode::Null),
        _ => Err(js_error("invalid stdio option")),
    }
}

fn options_protocol(
    options: &Option<JsValue>,
    key: &str,
    default: PortProtocol,
) -> Result<PortProtocol, JsValue> {
    let value = options_string(options, key)?;
    let Some(value) = value else {
        return Ok(default);
    };
    match value.as_str() {
        "http" => Ok(PortProtocol::Http),
        "https" => Ok(PortProtocol::Https),
        "tcp" => Ok(PortProtocol::Tcp),
        "udp" => Ok(PortProtocol::Udp),
        _ => Err(js_error("invalid port protocol")),
    }
}

fn array_to_strings(values: &js_sys::Array) -> Result<Vec<String>, JsValue> {
    let mut args = Vec::new();
    for value in values.iter() {
        let value = value
            .as_string()
            .ok_or_else(|| js_error("args must be strings"))?;
        args.push(value);
    }
    Ok(args)
}

fn js_to_mount_tree(value: JsValue) -> Result<MountTree, JsValue> {
    if value.is_string() {
        let data = value
            .as_string()
            .ok_or_else(|| js_error("string mount data must be utf-8"))?
            .into_bytes();
        return Ok(MountTree::File(MountFile {
            data,
            executable: false,
        }));
    }
    if value.is_instance_of::<Uint8Array>() {
        let data = Uint8Array::new(&value).to_vec();
        return Ok(MountTree::File(MountFile {
            data,
            executable: false,
        }));
    }
    if !value.is_object() {
        return Err(js_error(
            "mount entry must be an object, string, or Uint8Array",
        ));
    }
    let obj: Object = value
        .dyn_into()
        .map_err(|_| js_error("mount entry must be an object"))?;
    let file_value = Reflect::get(&obj, &JsValue::from_str("file"))?;
    if !file_value.is_undefined() && !file_value.is_null() {
        let data = if file_value.is_string() {
            file_value
                .as_string()
                .ok_or_else(|| js_error("mount file must be utf-8"))?
                .into_bytes()
        } else if file_value.is_instance_of::<Uint8Array>() {
            Uint8Array::new(&file_value).to_vec()
        } else {
            return Err(js_error("mount file must be a string or Uint8Array"));
        };
        let executable = options_bool(&Some(obj.into()), "executable", false)?;
        return Ok(MountTree::File(MountFile { data, executable }));
    }
    let keys = Object::keys(&obj);
    let mut children = BTreeMap::new();
    for key in keys.iter() {
        let name = key
            .as_string()
            .ok_or_else(|| js_error("mount key must be a string"))?;
        let child = Reflect::get(&obj, &key)?;
        children.insert(name, js_to_mount_tree(child)?);
    }
    Ok(MountTree::Directory(children))
}

fn file_type_label(file_type: FileType) -> &'static str {
    match file_type {
        FileType::File => "file",
        FileType::Directory => "dir",
        FileType::Symlink => "symlink",
    }
}

fn protocol_label(protocol: PortProtocol) -> &'static str {
    match protocol {
        PortProtocol::Http => "http",
        PortProtocol::Https => "https",
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    }
}

fn event_kind_label(kind: FsEventKind) -> &'static str {
    match kind {
        FsEventKind::Created => "created",
        FsEventKind::Modified => "modified",
        FsEventKind::Removed => "removed",
        FsEventKind::Renamed => "renamed",
    }
}

fn signal_to_int(signal: ProcessSignal) -> i32 {
    match signal {
        ProcessSignal::Kill => 9,
        ProcessSignal::Term => 15,
        ProcessSignal::Int => 2,
        ProcessSignal::Custom(value) => value,
    }
}

fn parse_signal(signal: Option<i32>) -> ProcessSignal {
    match signal {
        Some(9) => ProcessSignal::Kill,
        Some(15) => ProcessSignal::Term,
        Some(2) => ProcessSignal::Int,
        Some(value) => ProcessSignal::Custom(value),
        None => ProcessSignal::Term,
    }
}

fn err_to_js(err: WosixError) -> JsValue {
    js_sys::Error::new(&err.to_string()).into()
}

fn js_error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}
