/* tslint:disable */
/* eslint-disable */

export class FsHandle {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    mkdir(path: string, options?: any | null): void;
    mount(tree: any): void;
    readFile(path: string): Uint8Array;
    readdir(path: string): Array<any>;
    rename(from: string, to: string): void;
    rm(path: string, options?: any | null): void;
    stat(path: string): any;
    watch(path: string, options?: any | null): FsWatchHandle;
    writeFile(path: string, data: Uint8Array, options?: any | null): void;
}

export class FsWatchHandle {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    close(): void;
    nextEvent(): any;
}

export class ProcessHandle {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    close(): void;
    exit(): Promise<any>;
    kill(signal?: number | null): void;
    outputStream(): any;
    pid(): number;
    readOutput(max_bytes?: number | null): any;
    readStderr(max_bytes?: number | null): any;
    readStdout(max_bytes?: number | null): any;
    stderrStream(): any;
    stdinStream(): any;
    stdoutStream(): any;
    wait(): any;
    writeStdin(data: any): number;
}

export class WebContainer {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    static boot(): WebContainer;
    clearForegroundPid(): void;
    foregroundPid(): any;
    fs(): FsHandle;
    listProcesses(): any;
    nextPortEvent(): any;
    offPortEvent(id: number): void;
    onPortEvent(callback: Function): number;
    publishPort(port: number, options?: any | null): any;
    setForegroundPid(pid: number): void;
    signalForeground(signal?: number | null): boolean;
    spawn(program: string, args: Array<any>, options?: any | null): ProcessHandle;
    unpublishPort(port: number): void;
}

/**
 * Initialize the WASM bridge.
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_fshandle_free: (a: number, b: number) => void;
    readonly __wbg_fswatchhandle_free: (a: number, b: number) => void;
    readonly __wbg_processhandle_free: (a: number, b: number) => void;
    readonly __wbg_webcontainer_free: (a: number, b: number) => void;
    readonly fshandle_mkdir: (a: number, b: number, c: number, d: number) => [number, number];
    readonly fshandle_mount: (a: number, b: any) => [number, number];
    readonly fshandle_readFile: (a: number, b: number, c: number) => [number, number, number];
    readonly fshandle_readdir: (a: number, b: number, c: number) => [number, number, number];
    readonly fshandle_rename: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly fshandle_rm: (a: number, b: number, c: number, d: number) => [number, number];
    readonly fshandle_stat: (a: number, b: number, c: number) => [number, number, number];
    readonly fshandle_watch: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly fshandle_writeFile: (a: number, b: number, c: number, d: any, e: number) => [number, number];
    readonly fswatchhandle_close: (a: number) => [number, number];
    readonly fswatchhandle_nextEvent: (a: number) => [number, number, number];
    readonly processhandle_close: (a: number) => void;
    readonly processhandle_exit: (a: number) => any;
    readonly processhandle_kill: (a: number, b: number) => [number, number];
    readonly processhandle_outputStream: (a: number) => [number, number, number];
    readonly processhandle_pid: (a: number) => number;
    readonly processhandle_readOutput: (a: number, b: number) => [number, number, number];
    readonly processhandle_readStderr: (a: number, b: number) => [number, number, number];
    readonly processhandle_readStdout: (a: number, b: number) => [number, number, number];
    readonly processhandle_stderrStream: (a: number) => [number, number, number];
    readonly processhandle_stdinStream: (a: number) => [number, number, number];
    readonly processhandle_stdoutStream: (a: number) => [number, number, number];
    readonly processhandle_wait: (a: number) => [number, number, number];
    readonly processhandle_writeStdin: (a: number, b: any) => [number, number, number];
    readonly webcontainer_boot: () => number;
    readonly webcontainer_clearForegroundPid: (a: number) => void;
    readonly webcontainer_foregroundPid: (a: number) => any;
    readonly webcontainer_fs: (a: number) => number;
    readonly webcontainer_listProcesses: (a: number) => [number, number, number];
    readonly webcontainer_nextPortEvent: (a: number) => [number, number, number];
    readonly webcontainer_offPortEvent: (a: number, b: number) => void;
    readonly webcontainer_onPortEvent: (a: number, b: any) => number;
    readonly webcontainer_publishPort: (a: number, b: number, c: number) => [number, number, number];
    readonly webcontainer_setForegroundPid: (a: number, b: number) => [number, number];
    readonly webcontainer_signalForeground: (a: number, b: number) => [number, number, number];
    readonly webcontainer_spawn: (a: number, b: number, c: number, d: any, e: number) => [number, number, number];
    readonly webcontainer_unpublishPort: (a: number, b: number) => [number, number];
    readonly init: () => void;
    readonly wasm_bindgen__closure__destroy__h73676c04a155765f: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h5c04cc9d976e355f: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hf6802520b521944b: (a: number, b: number, c: any, d: any) => any;
    readonly wasm_bindgen__convert__closures_____invoke__h2b60c7780a70c97b: (a: number, b: number, c: any) => any;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
