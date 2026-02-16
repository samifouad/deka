const encoder = new TextEncoder();
const decoder = new TextDecoder();
let runtimeHost = null;
export function configurePhpRuntimeHost(host) {
    runtimeHost = host;
}
export function php_log(ptr, len) {
    const host = runtimeHost;
    if (!host)
        return;
    host.log(readString(ptr, len));
}
export function php_fs_read(pathPtr, pathLen, outPtr) {
    const host = runtimeHost;
    if (!host)
        return 0;
    const memory = host.getMemory();
    if (!memory)
        return 0;
    const path = readString(pathPtr, pathLen);
    const data = host.fsRead(path);
    if (!data)
        return 0;
    const bytes = data instanceof Uint8Array ? data : encoder.encode(String(data));
    writeWasmResultAt(memory, host.alloc, outPtr, bytes);
    return 1;
}
export function php_fs_exists(pathPtr, pathLen) {
    const host = runtimeHost;
    if (!host)
        return 0;
    const path = readString(pathPtr, pathLen);
    return host.fsExists(path) ? 1 : 0;
}
export function php_host_call(kindPtr, kindLen, actionPtr, actionLen, payloadPtr, payloadLen) {
    const host = runtimeHost;
    if (!host)
        return 0;
    const kind = readString(kindPtr, kindLen);
    const action = readString(actionPtr, actionLen);
    const payloadRaw = readString(payloadPtr, payloadLen);
    let payload = null;
    try {
        payload = payloadRaw.length ? JSON.parse(payloadRaw) : null;
    }
    catch {
        payload = null;
    }
    try {
        const value = host.hostCall(kind, action, payload);
        return writeWasmResult(host.alloc, value);
    }
    catch (err) {
        return writeWasmResult(host.alloc, {
            __deka_error: err instanceof Error ? err.message : String(err),
        });
    }
}
export function php_wasm_call(modulePtr, moduleLen, exportPtr, exportLen, payloadPtr, payloadLen) {
    const host = runtimeHost;
    if (!host)
        return 0;
    const moduleId = readString(modulePtr, moduleLen);
    const exportName = readString(exportPtr, exportLen);
    const payloadRaw = readString(payloadPtr, payloadLen);
    let payload = null;
    try {
        payload = payloadRaw.length ? JSON.parse(payloadRaw) : null;
    }
    catch {
        payload = null;
    }
    try {
        const value = host.wasmCall(moduleId, exportName, payload);
        return writeWasmResult(host.alloc, value);
    }
    catch (err) {
        return writeWasmResult(host.alloc, {
            __deka_error: err instanceof Error ? err.message : String(err),
        });
    }
}
function readString(ptr, len) {
    const host = runtimeHost;
    const memory = host?.getMemory();
    if (!host || !memory || ptr === 0 || len === 0) {
        return "";
    }
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    return decoder.decode(bytes);
}
function writeWasmResult(alloc, value) {
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
function writeWasmResultAt(memory, alloc, outPtr, bytes) {
    if (!outPtr)
        return;
    const payloadPtr = alloc(bytes.length);
    new Uint8Array(memory.buffer, payloadPtr, bytes.length).set(bytes);
    const view = new DataView(memory.buffer);
    view.setUint32(outPtr, payloadPtr >>> 0, true);
    view.setUint32(outPtr + 4, bytes.length >>> 0, true);
}
