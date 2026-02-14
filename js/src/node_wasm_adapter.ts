export type NodeWasmExports = {
  memory: WebAssembly.Memory;
  node_create(): number;
  node_spawn(handle: number, argc: number, argv_ptr: number): number;
  node_write_stdin(handle: number, ptr: number, len: number): number;
  node_read_stdout(handle: number, ptr: number, len: number): number;
  node_read_stderr(handle: number, ptr: number, len: number): number;
  node_read_output(handle: number, ptr: number, len: number): number;
  node_exit_code(handle: number): number;
  node_kill(handle: number, signal: number): void;
  node_close(handle: number): void;
};

export type NodeWasmAdapter = {
  exports: NodeWasmExports;
};
