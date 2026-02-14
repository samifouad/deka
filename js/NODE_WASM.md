# Node WASM Adapter

This file describes the expected adapter interface for wiring a real Node WASM
runtime into Adwa. The TypeScript interface lives in `js/src/node_wasm_adapter.ts`.

## Expected WASM exports
- `memory: WebAssembly.Memory`
- `node_create(): number`
- `node_spawn(handle: number, argc: number, argv_ptr: number): number`
- `node_write_stdin(handle: number, ptr: number, len: number): number`
- `node_read_stdout(handle: number, ptr: number, len: number): number`
- `node_read_stderr(handle: number, ptr: number, len: number): number`
- `node_read_output(handle: number, ptr: number, len: number): number`
- `node_exit_code(handle: number): number`
- `node_kill(handle: number, signal: number): void`
- `node_close(handle: number): void`

## Wiring TODO
- Define ABI for argv/env marshalling into WASM memory.
- Map Adwa FS calls to Node WASM imports (read/write/stat/readdir).
- Implement streaming I/O bridging between WASM pipes and JS `ReadableStream`.
- Track process lifecycles (exit code, signals, cleanup).
- Decide how to handle timers/event loop integration.
