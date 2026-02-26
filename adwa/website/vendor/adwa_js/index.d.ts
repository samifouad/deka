export type { NodeWasmAdapter, NodeWasmExports } from "./node_wasm_adapter.js";
import type { NodeWasmAdapter } from "./node_wasm_adapter.js";
export { DekaBrowserRuntime, DekaBrowserServer } from "./deka_runtime.js";
export type { Handler, ServeOptions, RuntimeFs } from "./deka_runtime.js";
export { createPhpHostBridge, PhpHostBridge } from "./phpx_host_bridge.js";
export type { BridgeCallInput, BridgeCallOutput, HostCapabilities, HostTarget, } from "./phpx_host_bridge.js";
export { PhpRuntimeAdapter, createPhpRuntimeAdapter, createPhpRuntimeAdapterFromBridgeOptions, } from "./phpx_runtime_adapter.js";
export type { PhpRunContext, PhpRunDiagnostic, PhpRunMode, PhpRunResult, PhpRuntimeAdapterOptions, } from "./phpx_runtime_adapter.js";
export { PhpRuntimeWasmExecutor, createPhpRuntimeWasmExecutor, } from "./phpx_wasm_executor.js";
export type { PhpRuntimeWasmExecutorOptions } from "./phpx_wasm_executor.js";
export type BootOptions = {
    init?: (module?: WebAssembly.Module | ArrayBuffer | Response) => Promise<unknown>;
    module?: WebAssembly.Module | ArrayBuffer | Response;
    nodeRuntime?: "shim" | "wasm";
    nodeWasm?: NodeWasmOptions;
};
export type NodeWasmOptions = {
    module?: WebAssembly.Module | ArrayBuffer | Response;
    url?: string;
    instantiate?: (module: WebAssembly.Module | ArrayBuffer | Response) => Promise<NodeWasmAdapter>;
    adapter?: NodeWasmAdapter;
};
export type PortEvent = {
    kind: "server-ready" | "port-closed";
    port: number;
    url?: string;
    protocol?: string;
};
export type AdwaBindings = {
    WebContainer: {
        boot(): AdwaWebContainer;
    };
    default?: (module?: WebAssembly.Module | ArrayBuffer | Response) => Promise<unknown>;
};
export type AdwaWebContainer = {
    fs(): AdwaFs;
    spawn(program: string, args: string[], options?: SpawnOptions): AdwaProcess;
    listProcesses(): Array<{
        pid: number;
        program: string;
        args: string[];
        cwd: string;
        ports: number[];
        running: boolean;
    }>;
    publishPort(port: number, options?: {
        protocol?: string;
        host?: string;
    }): {
        port: number;
        url: string;
        protocol: string;
    };
    unpublishPort(port: number): void;
    nextPortEvent(): PortEvent | null;
    onPortEvent(callback: (event: PortEvent) => void): number;
    offPortEvent(id: number): void;
};
export type AdwaFs = {
    readFile(path: string): Uint8Array;
    writeFile(path: string, data: Uint8Array, options?: WriteOptions): void;
    readdir(path: string): string[];
    mkdir(path: string, options?: MkdirOptions): void;
    rm(path: string, options?: RemoveOptions): void;
    rename(from: string, to: string): void;
    stat(path: string): {
        size: number;
        fileType: string;
    };
    mount(tree: MountTree): void;
    watch(path: string, options?: WatchOptions): AdwaFsWatchHandle;
};
export type AdwaFsWatchHandle = {
    nextEvent(): FsEvent | null;
    close(): void;
};
export type FsEvent = {
    path: string;
    kind: "created" | "modified" | "removed" | "renamed";
    targetPath?: string;
};
export type SpawnOptions = {
    cwd?: string;
    env?: Record<string, string>;
    clearEnv?: boolean;
    stdin?: "inherit" | "piped" | "null";
    stdout?: "inherit" | "piped" | "null";
    stderr?: "inherit" | "piped" | "null";
    pty?: boolean;
};
export type VirtualProcessResult = {
    code: number;
    signal?: number;
    stdout?: string | Uint8Array;
    stderr?: string | Uint8Array;
};
export type SpawnInterceptContext = {
    program: string;
    args: string[];
    options?: SpawnOptions;
    container: WebContainer;
};
export type SpawnInterceptor = ((context: SpawnInterceptContext) => AdwaProcess | null | Promise<AdwaProcess | null>) | null;
export type WriteOptions = {
    create?: boolean;
    truncate?: boolean;
};
export type MkdirOptions = {
    recursive?: boolean;
};
export type RemoveOptions = {
    recursive?: boolean;
    force?: boolean;
};
export type WatchOptions = {
    recursive?: boolean;
};
export type MountTree = string | Uint8Array | {
    file?: string | Uint8Array;
    executable?: boolean;
    [key: string]: MountTree | string | Uint8Array | boolean | undefined;
};
export declare class WebContainer {
    static boot(bindings: AdwaBindings, options?: BootOptions): Promise<WebContainer>;
    readonly fs: FileSystem;
    private readonly inner;
    private readonly innerFs;
    private nodeRuntime;
    private readonly listeners;
    private spawnInterceptor;
    private nextVirtualPid;
    private portSubscriptionId;
    private readonly portCallback;
    private constructor();
    private initNodeRuntime;
    mount(tree: MountTree): Promise<void>;
    spawn(program: string, args?: string[], options?: SpawnOptions): Promise<Process>;
    setSpawnInterceptor(interceptor: SpawnInterceptor): void;
    createVirtualProcess(runner: () => Promise<VirtualProcessResult> | VirtualProcessResult): AdwaProcess;
    listProcesses(): {
        pid: number;
        program: string;
        args: string[];
        cwd: string;
        ports: number[];
        running: boolean;
    }[];
    on(event: "server-ready" | "port" | "port-closed", listener: (event: PortEvent) => void): void;
    off(event: "server-ready" | "port" | "port-closed", listener: (event: PortEvent) => void): void;
    publishPort(port: number, options?: {
        protocol?: string;
        host?: string;
    }): {
        port: number;
        url: string;
        protocol: string;
    };
    unpublishPort(port: number): void;
    private ensurePortSubscription;
    private stopPortSubscriptionIfIdle;
    private dispatch;
}
export declare class FileSystem {
    private readonly inner;
    constructor(inner: AdwaFs);
    readFile(path: string): Promise<Uint8Array>;
    writeFile(path: string, data: Uint8Array, options?: WriteOptions): Promise<void>;
    readdir(path: string): Promise<string[]>;
    mkdir(path: string, options?: MkdirOptions): Promise<void>;
    rm(path: string, options?: RemoveOptions): Promise<void>;
    rename(from: string, to: string): Promise<void>;
    stat(path: string): Promise<{
        size: number;
        fileType: string;
    }>;
    mount(tree: MountTree): Promise<void>;
    watch(path: string, options?: WatchOptions): FsWatchHandle;
}
export declare class FsWatchHandle {
    private readonly inner;
    constructor(inner: AdwaFsWatchHandle);
    nextEvent(): Promise<FsEvent | null>;
    close(): void;
}
export type AdwaProcess = {
    pid(): number;
    wait(): {
        code: number;
        signal?: number;
    };
    exit(): Promise<{
        code: number;
        signal?: number;
    }>;
    writeStdin(data: string | Uint8Array): number;
    readStdout(maxBytes?: number): Uint8Array | null;
    readStderr(maxBytes?: number): Uint8Array | null;
    readOutput(maxBytes?: number): Uint8Array | null;
    stdinStream(): WritableStream<Uint8Array | string>;
    stdoutStream(): ReadableStream<Uint8Array>;
    stderrStream(): ReadableStream<Uint8Array>;
    outputStream(): ReadableStream<Uint8Array>;
    kill(signal?: number): void;
    close(): void;
};
export declare class Process {
    private readonly inner;
    readonly input: WritableStream<Uint8Array | string>;
    readonly output: ReadableStream<Uint8Array>;
    readonly stdout: ReadableStream<Uint8Array>;
    readonly stderr: ReadableStream<Uint8Array>;
    constructor(inner: AdwaProcess);
    get pid(): number;
    wait(): Promise<{
        code: number;
        signal?: number;
    }>;
    exit(): Promise<{
        code: number;
        signal?: number;
    }>;
    write(data: string | Uint8Array): Promise<number>;
    readStdout(maxBytes?: number): Promise<Uint8Array<ArrayBufferLike> | null>;
    readStderr(maxBytes?: number): Promise<Uint8Array<ArrayBufferLike> | null>;
    readOutput(maxBytes?: number): Promise<Uint8Array<ArrayBufferLike> | null>;
    kill(signal?: number): void;
    close(): void;
}
