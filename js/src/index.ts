export type { NodeWasmAdapter, NodeWasmExports } from "./node_wasm_adapter.js";
import type { NodeWasmAdapter, NodeWasmExports } from "./node_wasm_adapter.js";
export { DekaBrowserRuntime, DekaBrowserServer } from "./deka_runtime.js";
export type { Handler, ServeOptions, RuntimeFs } from "./deka_runtime.js";
export { createPhpHostBridge, PhpHostBridge } from "./phpx_host_bridge.js";
export type {
  BridgeCallInput,
  BridgeCallOutput,
  HostCapabilities,
  HostTarget,
} from "./phpx_host_bridge.js";
export {
  PhpRuntimeAdapter,
  createPhpRuntimeAdapter,
  createPhpRuntimeAdapterFromBridgeOptions,
} from "./phpx_runtime_adapter.js";
export type {
  PhpRunContext,
  PhpRunDiagnostic,
  PhpRunMode,
  PhpRunResult,
  PhpRuntimeAdapterOptions,
} from "./phpx_runtime_adapter.js";
export {
  PhpRuntimeWasmExecutor,
  createPhpRuntimeWasmExecutor,
} from "./phpx_wasm_executor.js";
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
  instantiate?: (
    module: WebAssembly.Module | ArrayBuffer | Response
  ) => Promise<NodeWasmAdapter>;
  adapter?: NodeWasmAdapter;
};

export type PortEvent = {
  kind: "server-ready" | "port-closed";
  port: number;
  url?: string;
  protocol?: string;
};

export type WosixBindings = {
  WebContainer: {
    boot(): WosixWebContainer;
  };
  default?: (module?: WebAssembly.Module | ArrayBuffer | Response) => Promise<unknown>;
};

export type WosixWebContainer = {
  fs(): WosixFs;
  spawn(program: string, args: string[], options?: SpawnOptions): WosixProcess;
  listProcesses(): Array<{
    pid: number;
    program: string;
    args: string[];
    cwd: string;
    ports: number[];
    running: boolean;
  }>;
  publishPort(port: number, options?: { protocol?: string; host?: string }): {
    port: number;
    url: string;
    protocol: string;
  };
  unpublishPort(port: number): void;
  nextPortEvent(): PortEvent | null;
  onPortEvent(callback: (event: PortEvent) => void): number;
  offPortEvent(id: number): void;
};

export type WosixFs = {
  readFile(path: string): Uint8Array;
  writeFile(path: string, data: Uint8Array, options?: WriteOptions): void;
  readdir(path: string): string[];
  mkdir(path: string, options?: MkdirOptions): void;
  rm(path: string, options?: RemoveOptions): void;
  rename(from: string, to: string): void;
  stat(path: string): { size: number; fileType: string };
  mount(tree: MountTree): void;
  watch(path: string, options?: WatchOptions): WosixFsWatchHandle;
};

export type WosixFsWatchHandle = {
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

export type SpawnInterceptor =
  | ((context: SpawnInterceptContext) => WosixProcess | null | Promise<WosixProcess | null>)
  | null;

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

export type MountTree =
  | string
  | Uint8Array
  | {
      file?: string | Uint8Array;
      executable?: boolean;
      [key: string]: MountTree | string | Uint8Array | boolean | undefined;
    };

export class WebContainer {
  static async boot(bindings: WosixBindings, options: BootOptions = {}): Promise<WebContainer> {
    const init = options.init ?? bindings.default;
    if (init) {
      await init(options.module);
    }
    const inner = bindings.WebContainer.boot();
    const container = new WebContainer(inner, options);
    await container.initNodeRuntime(options);
    return container;
  }

  readonly fs: FileSystem;
  private readonly inner: WosixWebContainer;
  private readonly innerFs: WosixFs;
  private nodeRuntime: NodeRuntime | null = null;
  private readonly listeners = new Map<string, Set<(event: PortEvent) => void>>();
  private spawnInterceptor: SpawnInterceptor = null;
  private nextVirtualPid = 50000;
  private portSubscriptionId: number | null = null;
  private readonly portCallback = (event: PortEvent) => {
    if (event.kind === "server-ready") {
      this.dispatch("server-ready", event);
      this.dispatch("port", event);
    } else if (event.kind === "port-closed") {
      this.dispatch("port-closed", event);
    }
  };

  private constructor(inner: WosixWebContainer, options: BootOptions) {
    this.inner = inner;
    this.innerFs = inner.fs();
    this.fs = new FileSystem(this.innerFs);
  }

  private async initNodeRuntime(options: BootOptions) {
    const mode = options.nodeRuntime ?? "shim";
    if (mode === "wasm") {
      const runtime = new NodeWasmRuntime(this.innerFs, options.nodeWasm);
      await runtime.init();
      this.nodeRuntime = runtime;
      return;
    }
    this.nodeRuntime = new NodeShimRuntime(this.innerFs);
  }

  async mount(tree: MountTree): Promise<void> {
    this.fs.mount(tree);
  }

  async spawn(program: string, args: string[] = [], options?: SpawnOptions): Promise<Process> {
    if (this.spawnInterceptor) {
      const intercepted = await this.spawnInterceptor({
        program,
        args,
        options,
        container: this,
      });
      if (intercepted) {
        return new Process(intercepted);
      }
    }
    if (isNodeProgram(program)) {
      if (!this.nodeRuntime) {
        this.nodeRuntime = new NodeShimRuntime(this.innerFs);
      }
      const handle = this.nodeRuntime.spawn(args, options);
      return new Process(handle);
    }
    const handle = this.inner.spawn(program, args, options);
    return new Process(handle);
  }

  setSpawnInterceptor(interceptor: SpawnInterceptor) {
    this.spawnInterceptor = interceptor ?? null;
  }

  createVirtualProcess(
    runner: () => Promise<VirtualProcessResult> | VirtualProcessResult
  ): WosixProcess {
    const pid = this.nextVirtualPid++;
    return new VirtualProcess(pid, runner);
  }

  listProcesses() {
    return this.inner.listProcesses();
  }

  on(event: "server-ready" | "port" | "port-closed", listener: (event: PortEvent) => void) {
    const set = this.listeners.get(event) ?? new Set();
    set.add(listener);
    this.listeners.set(event, set);
    this.ensurePortSubscription();
  }

  off(event: "server-ready" | "port" | "port-closed", listener: (event: PortEvent) => void) {
    const set = this.listeners.get(event);
    if (!set) {
      return;
    }
    set.delete(listener);
    if (set.size === 0) {
      this.listeners.delete(event);
    }
    this.stopPortSubscriptionIfIdle();
  }

  publishPort(port: number, options?: { protocol?: string; host?: string }) {
    return this.inner.publishPort(port, options);
  }

  unpublishPort(port: number) {
    this.inner.unpublishPort(port);
  }

  private ensurePortSubscription() {
    if (this.portSubscriptionId !== null) {
      return;
    }
    this.portSubscriptionId = this.inner.onPortEvent(this.portCallback);
  }

  private stopPortSubscriptionIfIdle() {
    if (this.listeners.size > 0) {
      return;
    }
    if (this.portSubscriptionId !== null) {
      this.inner.offPortEvent(this.portSubscriptionId);
      this.portSubscriptionId = null;
    }
  }

  private dispatch(event: string, payload: PortEvent) {
    const listeners = this.listeners.get(event);
    if (!listeners) {
      return;
    }
    for (const listener of listeners) {
      listener(payload);
    }
  }
}

export class FileSystem {
  private readonly inner: WosixFs;

  constructor(inner: WosixFs) {
    this.inner = inner;
  }

  async readFile(path: string): Promise<Uint8Array> {
    return this.inner.readFile(path);
  }

  async writeFile(path: string, data: Uint8Array, options?: WriteOptions): Promise<void> {
    this.inner.writeFile(path, data, options);
  }

  async readdir(path: string): Promise<string[]> {
    return this.inner.readdir(path);
  }

  async mkdir(path: string, options?: MkdirOptions): Promise<void> {
    this.inner.mkdir(path, options);
  }

  async rm(path: string, options?: RemoveOptions): Promise<void> {
    this.inner.rm(path, options);
  }

  async rename(from: string, to: string): Promise<void> {
    this.inner.rename(from, to);
  }

  async stat(path: string): Promise<{ size: number; fileType: string }> {
    return this.inner.stat(path);
  }

  async mount(tree: MountTree): Promise<void> {
    this.inner.mount(tree);
  }

  watch(path: string, options?: WatchOptions): FsWatchHandle {
    return new FsWatchHandle(this.inner.watch(path, options));
  }
}

export class FsWatchHandle {
  private readonly inner: WosixFsWatchHandle;

  constructor(inner: WosixFsWatchHandle) {
    this.inner = inner;
  }

  async nextEvent(): Promise<FsEvent | null> {
    return this.inner.nextEvent();
  }

  close() {
    this.inner.close();
  }
}

export type WosixProcess = {
  pid(): number;
  wait(): { code: number; signal?: number };
  exit(): Promise<{ code: number; signal?: number }>;
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

export class Process {
  private readonly inner: WosixProcess;
  readonly input: WritableStream<Uint8Array | string>;
  readonly output: ReadableStream<Uint8Array>;
  readonly stdout: ReadableStream<Uint8Array>;
  readonly stderr: ReadableStream<Uint8Array>;

  constructor(inner: WosixProcess) {
    this.inner = inner;
    this.input = inner.stdinStream();
    this.output = inner.outputStream();
    this.stdout = inner.stdoutStream();
    this.stderr = inner.stderrStream();
  }

  get pid(): number {
    return this.inner.pid();
  }

  async wait() {
    return this.inner.wait();
  }

  async exit() {
    return this.inner.exit();
  }

  async write(data: string | Uint8Array) {
    return this.inner.writeStdin(data);
  }

  async readStdout(maxBytes?: number) {
    return this.inner.readStdout(maxBytes);
  }

  async readStderr(maxBytes?: number) {
    return this.inner.readStderr(maxBytes);
  }

  async readOutput(maxBytes?: number) {
    return this.inner.readOutput(maxBytes);
  }

  kill(signal?: number) {
    this.inner.kill(signal);
  }

  close() {
    this.inner.close();
  }
}

type ExitStatus = { code: number; signal?: number };

interface NodeRuntime {
  init?(): Promise<void>;
  spawn(args: string[], options?: SpawnOptions): WosixProcess;
}

class NodeShimRuntime implements NodeRuntime {
  private readonly fs: WosixFs;
  private readonly encoder = new TextEncoder();
  private readonly decoder = new TextDecoder();
  private nextPid = 1000;

  constructor(fs: WosixFs) {
    this.fs = fs;
  }

  spawn(args: string[], options?: SpawnOptions): WosixProcess {
    const pid = this.nextPid++;
    const proc = new NodeShimProcess(pid);
    queueMicrotask(() => this.runProcess(proc, args, options));
    return proc;
  }

  private runProcess(proc: NodeShimProcess, args: string[], options?: SpawnOptions) {
    const projectRoot = normalizePath(options?.cwd ?? "/");
    const cwdRef = { value: projectRoot };
    const env = { ...(options?.env ?? {}) };
    const argv = ["node", ...args];
    const moduleCache = new Map<string, ModuleRecord>();
    const fsModule = createFsModule(this.fs, cwdRef, this.decoder, this.encoder);
    const pathModule = createPathModule(cwdRef);
    const process = {
      argv,
      env,
      cwd: () => cwdRef.value,
      chdir: (path: string) => {
        cwdRef.value = resolvePath(cwdRef.value, path);
      },
      exit: (code = 0) => {
        throw new ExitSignal(code);
      },
      stdout: {
        write: (chunk: unknown) => {
          proc.writeStdout(coerceBytes(chunk, this.encoder));
        },
      },
      stderr: {
        write: (chunk: unknown) => {
          proc.writeStderr(coerceBytes(chunk, this.encoder));
        },
      },
    };
    const console = makeConsole(proc, this.encoder);
    const require = createRequire({
      fs: this.fs,
      cwdRef,
      projectRoot,
      decoder: this.decoder,
      encoder: this.encoder,
      moduleCache,
      fsModule,
      pathModule,
      process,
      console,
    });
    try {
      if (args.length === 0) {
        proc.finish({ code: 0 });
        return;
      }
      if (args[0] === "-e" || args[0] === "--eval") {
        const code = args[1] ?? "";
        const module = { exports: {} as Record<string, unknown> };
        runModuleCode({
          code,
          filename: "<eval>",
          dirname: cwdRef.value,
          require,
          process,
          console,
          buffer: BufferShim,
          module,
        });
        proc.finish({ code: 0 });
        return;
      }
      const entry = resolvePath(cwdRef.value, args[0]);
      const filename = resolveModuleFile(this.fs, entry);
      const code = this.decoder.decode(this.fs.readFile(filename));
      const module = { exports: {} as Record<string, unknown> };
      runModuleCode({
        code,
        filename,
        dirname: dirname(filename),
        require,
        process,
        console,
        buffer: BufferShim,
        module,
      });
      proc.finish({ code: 0 });
    } catch (err) {
      if (err instanceof ExitSignal) {
        proc.finish({ code: err.code });
        return;
      }
      const message = err instanceof Error ? err.message : String(err);
      proc.writeStderr(this.encoder.encode(`${message}\n`));
      proc.finish({ code: 1 });
    }
  }
}

class NodeWasmRuntime implements NodeRuntime {
  private readonly fs: WosixFs;
  private readonly options: NodeWasmOptions | undefined;
  private initPromise: Promise<void> | null = null;
  private adapter: NodeWasmAdapter | null = null;

  constructor(fs: WosixFs, options?: NodeWasmOptions) {
    this.fs = fs;
    this.options = options;
  }

  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }
    this.initPromise = this.load();
    return this.initPromise;
  }

  spawn(_args: string[], _options?: SpawnOptions): WosixProcess {
    if (!this.adapter) {
      throw new Error(
        "Node WASM adapter is not loaded. Provide nodeWasm.adapter or nodeWasm.instantiate."
      );
    }
    throw new Error("Node WASM spawn not wired yet. See js/NODE_WASM.md.");
  }

  private async load(): Promise<void> {
    if (this.options?.adapter) {
      this.adapter = this.options.adapter;
      return;
    }
    let module = this.options?.module;
    if (!module && this.options?.url) {
      const response = await fetch(this.options.url);
      module = await response.arrayBuffer();
    }
    if (!module) {
      return;
    }
    if (module instanceof Response) {
      module = await module.arrayBuffer();
    }
    if (this.options?.instantiate) {
      this.adapter = await this.options.instantiate(module);
      return;
    }
    const result = (await WebAssembly.instantiate(module, {})) as
      | WebAssembly.Instance
      | WebAssembly.WebAssemblyInstantiatedSource;
    const instance =
      result instanceof WebAssembly.Instance ? result : result.instance;
    this.adapter = { exports: instance.exports as unknown as NodeWasmExports };
  }
}

class NodeShimProcess implements WosixProcess {
  private readonly stdoutQueue = new StreamQueue();
  private readonly stderrQueue = new StreamQueue();
  private readonly outputQueue = new StreamQueue();
  private readonly stdinQueue = new StdinQueue();
  private readonly exitPromise: Promise<ExitStatus>;
  private exitResolve: ((status: ExitStatus) => void) | null = null;
  private exitStatus: ExitStatus = { code: 0 };
  private closed = false;

  constructor(private readonly procId: number) {
    this.exitPromise = new Promise((resolve) => {
      this.exitResolve = resolve;
    });
  }

  pid(): number {
    return this.procId;
  }

  wait(): ExitStatus {
    return this.exitStatus;
  }

  exit(): Promise<ExitStatus> {
    return this.exitPromise;
  }

  writeStdin(data: string | Uint8Array): number {
    const bytes = coerceBytes(data, new TextEncoder());
    this.stdinQueue.push(bytes);
    return bytes.length;
  }

  readStdout(maxBytes?: number): Uint8Array | null {
    return this.stdoutQueue.read(maxBytes);
  }

  readStderr(maxBytes?: number): Uint8Array | null {
    return this.stderrQueue.read(maxBytes);
  }

  readOutput(maxBytes?: number): Uint8Array | null {
    return this.outputQueue.read(maxBytes);
  }

  stdinStream(): WritableStream<Uint8Array | string> {
    return this.stdinQueue.stream;
  }

  stdoutStream(): ReadableStream<Uint8Array> {
    return this.stdoutQueue.stream;
  }

  stderrStream(): ReadableStream<Uint8Array> {
    return this.stderrQueue.stream;
  }

  outputStream(): ReadableStream<Uint8Array> {
    return this.outputQueue.stream;
  }

  kill(signal?: number): void {
    if (this.closed) {
      return;
    }
    this.finish({ code: 128, signal });
  }

  close(): void {
    this.closed = true;
  }

  writeStdout(bytes: Uint8Array) {
    this.stdoutQueue.push(bytes);
    this.outputQueue.push(bytes);
  }

  writeStderr(bytes: Uint8Array) {
    this.stderrQueue.push(bytes);
    this.outputQueue.push(bytes);
  }

  finish(status: ExitStatus) {
    this.exitStatus = status;
    if (this.exitResolve) {
      this.exitResolve(status);
      this.exitResolve = null;
    }
  }
}

class VirtualProcess extends NodeShimProcess {
  constructor(
    procId: number,
    runner: () => Promise<VirtualProcessResult> | VirtualProcessResult
  ) {
    super(procId);
    queueMicrotask(async () => {
      try {
        const result = await runner();
        if (result?.stdout !== undefined) {
          this.writeStdout(coerceBytes(result.stdout, new TextEncoder()));
        }
        if (result?.stderr !== undefined) {
          this.writeStderr(coerceBytes(result.stderr, new TextEncoder()));
        }
        this.finish({
          code: Number.isFinite(result?.code) ? Number(result.code) : 0,
          signal: result?.signal,
        });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        this.writeStderr(new TextEncoder().encode(`${message}\n`));
        this.finish({ code: 1 });
      }
    });
  }
}

class StreamQueue {
  private readonly queue: Uint8Array[] = [];
  private controller: ReadableStreamDefaultController<Uint8Array> | null = null;
  readonly stream: ReadableStream<Uint8Array>;

  constructor() {
    this.stream = new ReadableStream<Uint8Array>({
      start: (controller) => {
        this.controller = controller;
      },
      pull: (controller) => {
        const chunk = this.queue.shift();
        if (chunk) {
          controller.enqueue(chunk);
        }
      },
    });
  }

  push(bytes: Uint8Array) {
    if (this.controller) {
      this.controller.enqueue(bytes);
      return;
    }
    this.queue.push(bytes);
  }

  read(maxBytes?: number): Uint8Array | null {
    const chunk = this.queue[0];
    if (!chunk) {
      return null;
    }
    if (!maxBytes || chunk.length <= maxBytes) {
      return this.queue.shift() ?? null;
    }
    const head = chunk.slice(0, maxBytes);
    this.queue[0] = chunk.slice(maxBytes);
    return head;
  }
}

class StdinQueue {
  private readonly queue: Uint8Array[] = [];
  readonly stream: WritableStream<Uint8Array | string>;

  constructor() {
    this.stream = new WritableStream<Uint8Array | string>({
      write: (chunk) => {
        const bytes = coerceBytes(chunk, new TextEncoder());
        this.queue.push(bytes);
      },
    });
  }

  push(bytes: Uint8Array) {
    this.queue.push(bytes);
  }
}

type ModuleRecord = {
  exports: Record<string, unknown> | unknown;
};

class ExitSignal extends Error {
  constructor(readonly code: number) {
    super("Process exit");
  }
}

function isNodeProgram(program: string): boolean {
  return program === "node" || program.endsWith("/node");
}

function createRequire(options: {
  fs: WosixFs;
  cwdRef: { value: string };
  projectRoot: string;
  decoder: TextDecoder;
  encoder: TextEncoder;
  moduleCache: Map<string, ModuleRecord>;
  fsModule: ReturnType<typeof createFsModule>;
  pathModule: ReturnType<typeof createPathModule>;
  process: Record<string, unknown>;
  console: Console;
}) {
  return function require(specifier: string): unknown {
    if (specifier === "fs") {
      return options.fsModule;
    }
    if (specifier === "path") {
      return options.pathModule;
    }
    if (specifier === "process") {
      return options.process;
    }
    if (specifier === "buffer") {
      return { Buffer: BufferShim };
    }
    const resolved = resolveRequireSpecifier(
      options.fs,
      options.cwdRef.value,
      options.projectRoot,
      specifier
    );
    const filename = resolveModuleFile(options.fs, resolved);
    const cached = options.moduleCache.get(filename);
    if (cached) {
      return cached.exports;
    }
    const record: ModuleRecord = { exports: {} };
    options.moduleCache.set(filename, record);
    const code = options.decoder.decode(options.fs.readFile(filename));
    if (filename.endsWith(".json")) {
      record.exports = JSON.parse(code);
      return record.exports;
    }
    const module = { exports: record.exports as Record<string, unknown> };
    const localRequire = createRequire({
      ...options,
      cwdRef: { value: dirname(filename) },
    });
    runModuleCode({
      code,
      filename,
      dirname: dirname(filename),
      require: localRequire,
      process: options.process,
      console: options.console,
      buffer: BufferShim,
      module,
    });
    record.exports = module.exports;
    return record.exports;
  };
}

function runModuleCode(options: {
  code: string;
  filename: string;
  dirname: string;
  require: (specifier: string) => unknown;
  process: Record<string, unknown>;
  console: Console;
  buffer: typeof BufferShim;
  module: { exports: Record<string, unknown> };
}) {
  const wrapper = new Function(
    "exports",
    "require",
    "module",
    "__filename",
    "__dirname",
    "process",
    "console",
    "Buffer",
    options.code
  );
  wrapper(
    options.module.exports,
    options.require,
    options.module,
    options.filename,
    options.dirname,
    options.process,
    options.console,
    options.buffer
  );
}

function createFsModule(
  fs: WosixFs,
  cwdRef: { value: string },
  decoder: TextDecoder,
  encoder: TextEncoder
) {
  return {
    readFileSync: (path: string, options?: { encoding?: string } | string) => {
      const resolved = resolvePath(cwdRef.value, path);
      const data = fs.readFile(resolved);
      const encoding = typeof options === "string" ? options : options?.encoding;
      if (encoding === "utf8" || encoding === "utf-8") {
        return decoder.decode(data);
      }
      return data;
    },
    writeFileSync: (path: string, data: string | Uint8Array, options?: WriteOptions) => {
      const resolved = resolvePath(cwdRef.value, path);
      const bytes = typeof data === "string" ? encoder.encode(data) : data;
      fs.writeFile(resolved, bytes, options);
    },
    readdirSync: (path: string) => {
      const resolved = resolvePath(cwdRef.value, path);
      return fs.readdir(resolved);
    },
    mkdirSync: (path: string, options?: MkdirOptions) => {
      const resolved = resolvePath(cwdRef.value, path);
      fs.mkdir(resolved, options);
    },
    rmSync: (path: string, options?: RemoveOptions) => {
      const resolved = resolvePath(cwdRef.value, path);
      fs.rm(resolved, options);
    },
    renameSync: (from: string, to: string) => {
      fs.rename(resolvePath(cwdRef.value, from), resolvePath(cwdRef.value, to));
    },
    statSync: (path: string) => {
      const resolved = resolvePath(cwdRef.value, path);
      const stat = fs.stat(resolved);
      return {
        size: stat.size,
        isFile: () => stat.fileType === "file",
        isDirectory: () => stat.fileType === "dir",
      };
    },
  };
}

function createPathModule(cwdRef: { value: string }) {
  return {
    join: (...parts: string[]) => normalizePath(parts.join("/")),
    resolve: (...parts: string[]) => {
      if (parts.length === 0) {
        return cwdRef.value;
      }
      const first = parts[0];
      const base = first.startsWith("/") ? "" : cwdRef.value;
      return normalizePath([base, ...parts].join("/"));
    },
    dirname,
    basename,
    extname,
    isAbsolute: (path: string) => path.startsWith("/"),
  };
}

function resolvePath(cwd: string, path: string): string {
  if (path.startsWith("/")) {
    return normalizePath(path);
  }
  return normalizePath(`${cwd}/${path}`);
}

function resolveRequireSpecifier(
  fs: WosixFs,
  cwd: string,
  projectRoot: string,
  specifier: string
): string {
  if (specifier.startsWith("/")) {
    return normalizePath(specifier);
  }
  if (specifier.startsWith("./") || specifier.startsWith("../")) {
    return resolvePath(cwd, specifier);
  }
  if (specifier.startsWith("@/")) {
    return normalizePath(`${projectRoot}/${specifier.slice(2)}`);
  }
  const moduleRoot = normalizePath(`${projectRoot}/php_modules/${specifier}`);
  return resolveBareModuleSpecifier({
    fs,
    moduleRoot,
    projectRoot,
    specifier,
  });
}

function resolveBareModuleSpecifier(options: {
  fs?: WosixFs;
  moduleRoot: string;
  projectRoot: string;
  specifier: string;
}): string {
  const { fs, moduleRoot, projectRoot, specifier } = options;
  if (!fs) {
    return moduleRoot;
  }
  if (hasModuleEntry(fs, moduleRoot)) {
    return moduleRoot;
  }
  const cacheRoot = normalizePath(`${projectRoot}/php_modules/.cache/${specifier}`);
  if (hasModuleEntry(fs, cacheRoot)) {
    return cacheRoot;
  }
  return moduleRoot;
}

function hasModuleEntry(fs: WosixFs, root: string): boolean {
  const candidates = [
    root,
    `${root}.js`,
    `${root}.json`,
    `${root}/index.js`,
    `${root}/index.json`,
  ];
  for (const candidate of candidates) {
    if (fileType(fs, candidate) === "file") {
      return true;
    }
  }
  return false;
}

function resolveModuleFile(fs: WosixFs, path: string): string {
  const candidates = [
    path,
    `${path}.js`,
    `${path}.json`,
    `${path}/index.js`,
    `${path}/index.json`,
  ];
  for (const candidate of candidates) {
    if (fileType(fs, candidate) === "file") {
      return candidate;
    }
  }
  throw new Error(`Module not found: ${path}`);
}

function fileType(fs: WosixFs, path: string): "file" | "dir" | null {
  try {
    const stat = fs.stat(path);
    if (stat.fileType === "file") {
      return "file";
    }
    if (stat.fileType === "dir") {
      return "dir";
    }
    return null;
  } catch {
    return null;
  }
}

function normalizePath(path: string): string {
  const absolute = path.startsWith("/");
  const parts = path.split("/").filter((part) => part.length > 0);
  const stack: string[] = [];
  for (const part of parts) {
    if (part === ".") {
      continue;
    }
    if (part === "..") {
      stack.pop();
      continue;
    }
    stack.push(part);
  }
  return `${absolute ? "/" : ""}${stack.join("/")}` || "/";
}

function dirname(path: string): string {
  if (path === "/") {
    return "/";
  }
  const parts = path.split("/").filter((part) => part.length > 0);
  parts.pop();
  return parts.length === 0 ? "/" : `/${parts.join("/")}`;
}

function basename(path: string): string {
  const parts = path.split("/").filter((part) => part.length > 0);
  return parts[parts.length - 1] ?? "";
}

function extname(path: string): string {
  const base = basename(path);
  const index = base.lastIndexOf(".");
  if (index <= 0) {
    return "";
  }
  return base.slice(index);
}

function coerceBytes(input: unknown, encoder: TextEncoder): Uint8Array {
  if (input instanceof Uint8Array) {
    return input;
  }
  if (typeof input === "string") {
    return encoder.encode(input);
  }
  return encoder.encode(String(input));
}

function makeConsole(proc: NodeShimProcess, encoder: TextEncoder): Console {
  const print = (writer: (bytes: Uint8Array) => void) => {
    return (...args: unknown[]) => {
      const message = args.map(String).join(" ");
      writer(encoder.encode(`${message}\n`));
    };
  };
  return {
    log: print((bytes) => proc.writeStdout(bytes)),
    info: print((bytes) => proc.writeStdout(bytes)),
    warn: print((bytes) => proc.writeStderr(bytes)),
    error: print((bytes) => proc.writeStderr(bytes)),
  } as Console;
}

class BufferShim {
  static from(value: string | Uint8Array | ArrayBuffer): Uint8Array {
    if (typeof value === "string") {
      return new TextEncoder().encode(value);
    }
    if (value instanceof Uint8Array) {
      return value;
    }
    return new Uint8Array(value);
  }
}
