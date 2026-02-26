type RuntimeHost = {
    getMemory(): WebAssembly.Memory | null;
    alloc(size: number): number;
    fsRead(path: string): Uint8Array | null;
    fsExists(path: string): boolean;
    hostCall(kind: string, action: string, payload: unknown): unknown;
    wasmCall(moduleId: string, exportName: string, payload: unknown): unknown;
    log(message: string): void;
};
export declare function configurePhpRuntimeHost(host: RuntimeHost | null): void;
export declare function php_log(ptr: number, len: number): void;
export declare function php_fs_read(pathPtr: number, pathLen: number, outPtr: number): number;
export declare function php_fs_exists(pathPtr: number, pathLen: number): number;
export declare function php_host_call(kindPtr: number, kindLen: number, actionPtr: number, actionLen: number, payloadPtr: number, payloadLen: number): number;
export declare function php_wasm_call(modulePtr: number, moduleLen: number, exportPtr: number, exportLen: number, payloadPtr: number, payloadLen: number): number;
export {};
