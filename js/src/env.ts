type RuntimeHost = {
  getMemory(): WebAssembly.Memory | null;
  alloc(size: number): number;
  fsRead(path: string): Uint8Array | null;
  fsExists(path: string): boolean;
  hostCall(kind: string, action: string, payload: unknown): unknown;
  wasmCall(moduleId: string, exportName: string, payload: unknown): unknown;
  log(message: string): void;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let runtimeHost: RuntimeHost | null = null;

export function configurePhpRuntimeHost(host: RuntimeHost | null): void {
  runtimeHost = host;
}

export function php_log(ptr: number, len: number): void {
  const host = runtimeHost;
  if (!host) return;
  host.log(readString(ptr, len));
}

export function php_fs_read(pathPtr: number, pathLen: number, outPtr: number): number {
  const host = runtimeHost;
  if (!host) return 0;
  const memory = host.getMemory();
  if (!memory) return 0;

  const path = readString(pathPtr, pathLen);
  const data = host.fsRead(path);
  if (!data) return 0;

  const bytes = data instanceof Uint8Array ? data : encoder.encode(String(data));
  writeWasmResultAt(memory, host.alloc, outPtr, bytes);
  return 1;
}

export function php_fs_exists(pathPtr: number, pathLen: number): number {
  const host = runtimeHost;
  if (!host) return 0;
  const path = readString(pathPtr, pathLen);
  return host.fsExists(path) ? 1 : 0;
}

export function php_host_call(
  kindPtr: number,
  kindLen: number,
  actionPtr: number,
  actionLen: number,
  payloadPtr: number,
  payloadLen: number
): number {
  const host = runtimeHost;
  if (!host) return 0;
  const kind = readString(kindPtr, kindLen);
  const action = readString(actionPtr, actionLen);
  const payloadRaw = readString(payloadPtr, payloadLen);
  let payload: unknown = null;
  try {
    payload = payloadRaw.length ? JSON.parse(payloadRaw) : null;
  } catch {
    payload = null;
  }

  try {
    const value = host.hostCall(kind, action, payload);
    return writeWasmResult(host.alloc, value);
  } catch (err) {
    return writeWasmResult(host.alloc, {
      __deka_error: err instanceof Error ? err.message : String(err),
    });
  }
}

export function php_wasm_call(
  modulePtr: number,
  moduleLen: number,
  exportPtr: number,
  exportLen: number,
  payloadPtr: number,
  payloadLen: number
): number {
  const host = runtimeHost;
  if (!host) return 0;
  const moduleId = readString(modulePtr, moduleLen);
  const exportName = readString(exportPtr, exportLen);
  const payloadRaw = readString(payloadPtr, payloadLen);
  let payload: unknown = null;
  try {
    payload = payloadRaw.length ? JSON.parse(payloadRaw) : null;
  } catch {
    payload = null;
  }

  try {
    const value = host.wasmCall(moduleId, exportName, payload);
    return writeWasmResult(host.alloc, value);
  } catch (err) {
    return writeWasmResult(host.alloc, {
      __deka_error: err instanceof Error ? err.message : String(err),
    });
  }
}

function readString(ptr: number, len: number): string {
  const host = runtimeHost;
  const memory = host?.getMemory();
  if (!host || !memory || ptr === 0 || len === 0) {
    return "";
  }
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  return decoder.decode(bytes);
}

function writeWasmResult(alloc: (size: number) => number, value: unknown): number {
  const bytes = encoder.encode(JSON.stringify(value ?? null));
  const payloadPtr = alloc(bytes.length);
  const host = runtimeHost;
  const memory = host?.getMemory();
  if (!host || !memory) {
    return 0;
  }
  new Uint8Array(memory.buffer, payloadPtr, bytes.length).set(bytes);

  const resultPtr = alloc(8);
  const view = new DataView(memory.buffer);
  view.setUint32(resultPtr, payloadPtr >>> 0, true);
  view.setUint32(resultPtr + 4, bytes.length >>> 0, true);
  return resultPtr >>> 0;
}

function writeWasmResultAt(
  memory: WebAssembly.Memory,
  alloc: (size: number) => number,
  outPtr: number,
  bytes: Uint8Array
): void {
  if (!outPtr) return;
  const payloadPtr = alloc(bytes.length);
  new Uint8Array(memory.buffer, payloadPtr, bytes.length).set(bytes);
  const view = new DataView(memory.buffer);
  view.setUint32(outPtr, payloadPtr >>> 0, true);
  view.setUint32(outPtr + 4, bytes.length >>> 0, true);
}
