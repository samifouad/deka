// Browser shim for wasm-bindgen generated modules that import bare "env".
// Keep this minimal and explicit; extend only when runtime imports require it.
export function php_log(_ptr: number, _len: number): void {
  // no-op logger in browser demo mode
}

export function php_fs_read(_path_ptr: number, _path_len: number): number {
  return 0;
}

export function php_fs_exists(_path_ptr: number, _path_len: number): number {
  return 0;
}

export function php_host_call(
  _kind_ptr: number,
  _kind_len: number,
  _action_ptr: number,
  _action_len: number,
  _payload_ptr: number,
  _payload_len: number
): number {
  return 0;
}

export function php_wasm_call(
  _module_ptr: number,
  _module_len: number,
  _func_ptr: number,
  _func_len: number,
  _payload_ptr: number,
  _payload_len: number
): number {
  return 0;
}
