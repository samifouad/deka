# WebContainers API Mapping

This document maps the public WebContainers JS API surface to the current
`wosix-core` traits. Gaps are called out explicitly.

## Container lifecycle
- `WebContainer.boot()` -> `wosix-wasm` bootstraps a `Host` with `FileSystem`,
  `ProcessHost`, and `NetHost` adapters.
- `webcontainer.mount(tree)` -> `FileSystem::mount_tree` with `MountTree`.

## File system
- `fs.readFile(path)` -> `FileSystem::read_file`
- `fs.writeFile(path, data, opts)` -> `FileSystem::write_file`
- `fs.readdir(path)` -> `FileSystem::readdir`
- `fs.mkdir(path, opts)` -> `FileSystem::mkdir`
- `fs.rm(path, opts)` -> `FileSystem::remove`
- `fs.rename(from, to)` -> `FileSystem::rename`
- `fs.stat(path)` -> `FileSystem::stat`
- `fs.watch(path, opts)` -> `FileSystem::watch` returning `FsWatcher` events
  (in-memory backend queues events, non-blocking `next_event`)

### wosix-wasm JS facade (current)
- `WebContainer.boot()` returns an in-memory container.
- `WebContainer.fs()` returns `FsHandle` with `readFile`, `writeFile`, `readdir`,
  `mkdir`, `rm`, `rename`, `stat`, `mount`, `watch`.
- `WebContainer.spawn()` returns a `ProcessHandle` with `pid`, `wait`, `exit`,
  `writeStdin`, `readStdout`, `readStderr`, `readOutput`, `stdinStream`,
  `stdoutStream`, `stderrStream`, `outputStream`, `kill`.
- `WebContainer.publishPort()`/`unpublishPort()` emit port events and
  `WebContainer.onPortEvent()` delivers them without polling.
- `FsWatchHandle.nextEvent()` returns an event object or `null` when drained.

### wosix-js wrapper (current)
- `WebContainer.boot()` is async and accepts wasm bindings + init loader.
- `WebContainer.on()` registers a wasm callback and emits `server-ready`/`port` events.
- `fs` and process methods return Promises for WebContainers-like ergonomics.
- `process.input`/`process.output` expose JS `WritableStream`/`ReadableStream` objects.
- `spawn("node", ...)` uses a minimal Node-like shim (CommonJS + fs/path/process).
- `nodeRuntime: "wasm"` is a placeholder for a real Node WASM integration.

### Current watcher semantics
- `Created`: new file/dir created (including `mount_tree` root).
- `Modified`: writes to existing file.
- `Removed`: successful delete.
- `Renamed`: emitted with `target_path` set to the destination.
- `WatchOptions.recursive = false` receives events for the watched path and
  its immediate children only.

## Processes
- `webcontainer.spawn(cmd, args, opts)` -> `ProcessHost::spawn(Command, SpawnOptions)`
- `process.exit` -> `ProcessHandle::wait`
- `process.kill(signal)` -> `ProcessHandle::kill`
- `process.input` -> `ProcessHandle::stdin`
- `process.output` -> JS adapter combines `stdout` + `stderr` streams

## Ports & networking
- `webcontainer.on('server-ready', (port, url))` -> `NetHost::next_event` returning
  `PortEvent::ServerReady(PortInfo)`
- `webcontainer.on('port', (port, protocol, url))` -> same stream; `PortInfo.protocol`
  carries the port type

## Gaps to close
- Align event semantics with WebContainers (debounce, rename behavior, recursive flag).
- PTY sizing info for `spawn` (cols/rows) beyond the `pty` boolean.
- Stream backpressure and blocking semantics for process I/O.
- Port events should be derived from actual listeners rather than manual publish.
