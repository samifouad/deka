export { DekaBrowserRuntime, DekaBrowserServer } from "./deka_runtime.js";
export { createPhpHostBridge, PhpHostBridge } from "./phpx_host_bridge.js";
export { PhpRuntimeAdapter, createPhpRuntimeAdapter, createPhpRuntimeAdapterFromBridgeOptions, } from "./phpx_runtime_adapter.js";
export { PhpRuntimeWasmExecutor, createPhpRuntimeWasmExecutor, } from "./phpx_wasm_executor.js";
export function createDekaWasmCommandRuntime(options) {
    let runtimePromise = null;
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    const ensureRuntime = async () => {
        if (runtimePromise) {
            return runtimePromise;
        }
        runtimePromise = (async () => {
            const bytes = options.wasmBytes instanceof Uint8Array
                ? options.wasmBytes.buffer.slice(options.wasmBytes.byteOffset, options.wasmBytes.byteOffset + options.wasmBytes.byteLength)
                : options.wasmBytes ??
                    (await (async () => {
                        const response = await fetch(options.wasmUrl);
                        if (!response.ok) {
                            throw new Error(`failed to load deka wasm cli: ${response.status}`);
                        }
                        return response.arrayBuffer();
                    })());
            const { instance } = await WebAssembly.instantiate(bytes, {});
            const exports = instance.exports;
            if (!exports.memory ||
                typeof exports.deka_wasm_alloc !== "function" ||
                typeof exports.deka_wasm_free !== "function" ||
                typeof exports.deka_wasm_run_json !== "function") {
                throw new Error("invalid deka wasm cli exports");
            }
            return exports;
        })();
        return runtimePromise;
    };
    return async (args) => {
        const runtime = await ensureRuntime();
        const payload = encoder.encode(JSON.stringify({ args }));
        const inPtr = runtime.deka_wasm_alloc(payload.length) >>> 0;
        new Uint8Array(runtime.memory.buffer).set(payload, inPtr);
        const packed = runtime.deka_wasm_run_json(inPtr, payload.length);
        runtime.deka_wasm_free(inPtr, payload.length);
        const outPtr = Number(packed & 0xffffffffn);
        const outLen = Number((packed >> 32n) & 0xffffffffn);
        const outBytes = new Uint8Array(runtime.memory.buffer, outPtr, outLen);
        const outJson = decoder.decode(outBytes);
        runtime.deka_wasm_free(outPtr, outLen);
        let parsed = {};
        try {
            parsed = JSON.parse(outJson);
        }
        catch {
            parsed = {
                code: 1,
                output: `failed to decode deka wasm output: ${outJson}`,
            };
        }
        return {
            code: parsed.code ?? 1,
            stdout: parsed.output ?? "",
        };
    };
}
export function createDekaBrowserCommandRuntime(options) {
    const decoder = new TextDecoder();
    return async (args, spawnOptions, context) => {
        const cliArgs = normalizeDekaArgs(args);
        const subcommand = cliArgs[0] ?? "";
        if (!subcommand || subcommand === "help" || subcommand === "--help") {
            return {
                code: 0,
                stdout: [
                    "deka (browser runtime)",
                    "",
                    "Usage:",
                    "  deka run [file]",
                    "  deka serve [file] [--port N]",
                    "  deka --version",
                ].join("\n"),
            };
        }
        if (subcommand === "--version" || subcommand === "-v" || subcommand === "version") {
            return {
                code: 0,
                stdout: "deka [browser runtime]\n",
            };
        }
        if (subcommand === "run") {
            const entry = normalizePath(cliArgs[1] ?? options.defaultRunEntry ?? "/main.phpx");
            return runPhpxEntry({
                context,
                decoder,
                phpRuntime: options.phpRuntime,
                entry,
            });
        }
        if (subcommand === "serve") {
            const cwd = normalizePath(spawnOptions?.cwd ?? "/");
            const projectRoot = normalizePath(options.projectRoot ?? "/");
            const config = readDekaJson(context.fs, projectRoot, decoder);
            const { entryArg, portArg } = parseServeArgs(cliArgs.slice(1));
            const entry = normalizePath(entryArg ??
                config?.serve?.entry ??
                options.defaultServeEntry ??
                options.defaultRunEntry ??
                "/main.phpx");
            const mode = config?.serve?.mode === "phpx" || config?.serve?.mode === "php"
                ? config.serve.mode
                : entry.endsWith(".phpx")
                    ? "phpx"
                    : "php";
            const port = portArg ??
                (typeof config?.serve?.port === "number" ? config.serve.port : undefined) ??
                options.defaultServePort ??
                8530;
            const initial = await runPhpxEntry({
                context,
                decoder,
                phpRuntime: options.phpRuntime,
                entry,
                mode,
            });
            if (initial.code !== 0) {
                return {
                    code: initial.code,
                    stdout: initial.stdout,
                    stderr: initial.stderr,
                };
            }
            const output = [
                "Booting PHPX runtime...",
                "PHPX diagnostics: wasm mode",
                `[handler] loaded ${entry} [mode=${mode}]`,
            ];
            let listenUrl = `http://localhost:${port}`;
            if (context.publishPort) {
                const info = context.publishPort(port, { protocol: "http" });
                if (info?.url) {
                    listenUrl = info.url;
                }
            }
            output.push(`[listen] ${listenUrl}`);
            if (context.writeStdout) {
                context.writeStdout(`${output.join("\n")}\n`);
            }
            if (context.waitForKill) {
                const status = await context.waitForKill();
                if (context.unpublishPort) {
                    context.unpublishPort(port);
                }
                return {
                    code: status.code,
                    signal: status.signal,
                };
            }
            return {
                code: 0,
                stdout: `${output.join("\n")}\n`,
            };
        }
        return options.cliRuntime(cliArgs, spawnOptions, context);
    };
}
export class WebContainer {
    static async boot(bindings, options = {}) {
        const init = options.init ?? bindings.default;
        if (init) {
            await init(options.module);
        }
        const inner = bindings.WebContainer.boot();
        const container = new WebContainer(inner, options);
        await container.initNodeRuntime(options);
        return container;
    }
    constructor(inner, options) {
        this.nodeRuntime = null;
        this.listeners = new Map();
        this.portSubscriptionId = null;
        this.portCallback = (event) => {
            if (event.kind === "server-ready") {
                this.dispatch("server-ready", event);
                this.dispatch("port", event);
            }
            else if (event.kind === "port-closed") {
                this.dispatch("port-closed", event);
            }
        };
        this.inner = inner;
        this.innerFs = inner.fs();
        this.fs = new FileSystem(this.innerFs);
        this.commandRuntimes = options.commandRuntimes ?? {};
    }
    async initNodeRuntime(options) {
        const mode = options.nodeRuntime ?? "shim";
        if (mode === "wasm") {
            const runtime = new NodeWasmRuntime(this.innerFs, options.nodeWasm);
            await runtime.init();
            this.nodeRuntime = runtime;
            return;
        }
        this.nodeRuntime = new NodeShimRuntime(this.innerFs);
    }
    async mount(tree) {
        this.fs.mount(tree);
    }
    async spawn(program, args = [], options) {
        const resolved = resolveSpawnTarget({
            fs: this.innerFs,
            commandRuntimes: this.commandRuntimes,
            cwd: options?.cwd ?? "/",
            env: options?.env ?? {},
            program,
            args,
        });
        if (resolved.kind === "not-found") {
            return createCommandNotFoundProcess(program);
        }
        if (resolved.kind === "runtime") {
            const runtime = resolved.runtime;
            const handle = new HostRuntimeProcess();
            queueMicrotask(async () => {
                try {
                    const result = await runtime(resolved.args, options, {
                        fs: this.innerFs,
                        publishPort: (port, publishOptions) => this.publishPort(port, publishOptions),
                        unpublishPort: (port) => this.unpublishPort(port),
                        waitForKill: () => handle.waitForKill(),
                        writeStdout: (data) => handle.writeStdout(coerceBytes(data, new TextEncoder())),
                        writeStderr: (data) => handle.writeStderr(coerceBytes(data, new TextEncoder())),
                    });
                    if (result.stdout) {
                        handle.writeStdout(coerceBytes(result.stdout, new TextEncoder()));
                    }
                    if (result.stderr) {
                        handle.writeStderr(coerceBytes(result.stderr, new TextEncoder()));
                    }
                    handle.finish({
                        code: result.code ?? 0,
                        signal: result.signal,
                    });
                }
                catch (err) {
                    const message = err instanceof Error ? err.message : String(err);
                    handle.writeStderr(new TextEncoder().encode(`${message}\n`));
                    handle.finish({ code: 1 });
                }
            });
            return new Process(handle);
        }
        if (isNodeProgram(resolved.program)) {
            if (!this.nodeRuntime) {
                this.nodeRuntime = new NodeShimRuntime(this.innerFs);
            }
            const handle = this.nodeRuntime.spawn(resolved.args, options);
            return new Process(handle);
        }
        const handle = this.inner.spawn(resolved.program, resolved.args, options);
        return new Process(handle);
    }
    on(event, listener) {
        const set = this.listeners.get(event) ?? new Set();
        set.add(listener);
        this.listeners.set(event, set);
        this.ensurePortSubscription();
    }
    off(event, listener) {
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
    publishPort(port, options) {
        return this.inner.publishPort(port, options);
    }
    unpublishPort(port) {
        this.inner.unpublishPort(port);
    }
    ensurePortSubscription() {
        if (this.portSubscriptionId !== null) {
            return;
        }
        this.portSubscriptionId = this.inner.onPortEvent(this.portCallback);
    }
    stopPortSubscriptionIfIdle() {
        if (this.listeners.size > 0) {
            return;
        }
        if (this.portSubscriptionId !== null) {
            this.inner.offPortEvent(this.portSubscriptionId);
            this.portSubscriptionId = null;
        }
    }
    dispatch(event, payload) {
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
    constructor(inner) {
        this.inner = inner;
    }
    async readFile(path) {
        return this.inner.readFile(path);
    }
    async writeFile(path, data, options) {
        this.inner.writeFile(path, data, options);
    }
    async readdir(path) {
        return this.inner.readdir(path);
    }
    async mkdir(path, options) {
        this.inner.mkdir(path, options);
    }
    async rm(path, options) {
        this.inner.rm(path, options);
    }
    async rename(from, to) {
        this.inner.rename(from, to);
    }
    async stat(path) {
        return this.inner.stat(path);
    }
    async mount(tree) {
        this.inner.mount(tree);
    }
    watch(path, options) {
        return new FsWatchHandle(this.inner.watch(path, options));
    }
}
export class FsWatchHandle {
    constructor(inner) {
        this.inner = inner;
    }
    async nextEvent() {
        return this.inner.nextEvent();
    }
    close() {
        this.inner.close();
    }
}
export class Process {
    constructor(inner) {
        this.inner = inner;
        this.input = inner.stdinStream();
        this.output = inner.outputStream();
        this.stdout = inner.stdoutStream();
        this.stderr = inner.stderrStream();
    }
    get pid() {
        return this.inner.pid();
    }
    async wait() {
        return this.inner.wait();
    }
    async exit() {
        return this.inner.exit();
    }
    async write(data) {
        return this.inner.writeStdin(data);
    }
    async readStdout(maxBytes) {
        return this.inner.readStdout(maxBytes);
    }
    async readStderr(maxBytes) {
        return this.inner.readStderr(maxBytes);
    }
    async readOutput(maxBytes) {
        return this.inner.readOutput(maxBytes);
    }
    kill(signal) {
        this.inner.kill(signal);
    }
    close() {
        this.inner.close();
    }
}
class NodeShimRuntime {
    constructor(fs) {
        this.encoder = new TextEncoder();
        this.decoder = new TextDecoder();
        this.nextPid = 1000;
        this.fs = fs;
    }
    spawn(args, options) {
        const pid = this.nextPid++;
        const proc = new NodeShimProcess(pid);
        queueMicrotask(() => this.runProcess(proc, args, options));
        return proc;
    }
    runProcess(proc, args, options) {
        const projectRoot = normalizePath(options?.cwd ?? "/");
        const cwdRef = { value: projectRoot };
        const env = { ...(options?.env ?? {}) };
        const argv = ["node", ...args];
        const moduleCache = new Map();
        const fsModule = createFsModule(this.fs, cwdRef, this.decoder, this.encoder);
        const pathModule = createPathModule(cwdRef);
        const process = {
            argv,
            env,
            cwd: () => cwdRef.value,
            chdir: (path) => {
                cwdRef.value = resolvePath(cwdRef.value, path);
            },
            exit: (code = 0) => {
                throw new ExitSignal(code);
            },
            stdout: {
                write: (chunk) => {
                    proc.writeStdout(coerceBytes(chunk, this.encoder));
                },
            },
            stderr: {
                write: (chunk) => {
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
                const module = { exports: {} };
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
            const module = { exports: {} };
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
        }
        catch (err) {
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
class NodeWasmRuntime {
    constructor(fs, options) {
        this.initPromise = null;
        this.adapter = null;
        this.fs = fs;
        this.options = options;
    }
    async init() {
        if (this.initPromise) {
            return this.initPromise;
        }
        this.initPromise = this.load();
        return this.initPromise;
    }
    spawn(_args, _options) {
        if (!this.adapter) {
            throw new Error("Node WASM adapter is not loaded. Provide nodeWasm.adapter or nodeWasm.instantiate.");
        }
        throw new Error("Node WASM spawn not wired yet. See js/NODE_WASM.md.");
    }
    async load() {
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
        const result = (await WebAssembly.instantiate(module, {}));
        const instance = result instanceof WebAssembly.Instance ? result : result.instance;
        this.adapter = { exports: instance.exports };
    }
}
class NodeShimProcess {
    constructor(procId) {
        this.procId = procId;
        this.stdoutQueue = new StreamQueue();
        this.stderrQueue = new StreamQueue();
        this.outputQueue = new StreamQueue();
        this.stdinQueue = new StdinQueue();
        this.exitResolve = null;
        this.exitStatus = { code: 0 };
        this.closed = false;
        this.exitPromise = new Promise((resolve) => {
            this.exitResolve = resolve;
        });
    }
    pid() {
        return this.procId;
    }
    wait() {
        return this.exitStatus;
    }
    exit() {
        return this.exitPromise;
    }
    writeStdin(data) {
        const bytes = coerceBytes(data, new TextEncoder());
        this.stdinQueue.push(bytes);
        return bytes.length;
    }
    readStdout(maxBytes) {
        return this.stdoutQueue.read(maxBytes);
    }
    readStderr(maxBytes) {
        return this.stderrQueue.read(maxBytes);
    }
    readOutput(maxBytes) {
        return this.outputQueue.read(maxBytes);
    }
    stdinStream() {
        return this.stdinQueue.stream;
    }
    stdoutStream() {
        return this.stdoutQueue.stream;
    }
    stderrStream() {
        return this.stderrQueue.stream;
    }
    outputStream() {
        return this.outputQueue.stream;
    }
    kill(signal) {
        if (this.closed) {
            return;
        }
        this.finish({ code: 128, signal });
    }
    close() {
        this.closed = true;
    }
    writeStdout(bytes) {
        this.stdoutQueue.push(bytes);
        this.outputQueue.push(bytes);
    }
    writeStderr(bytes) {
        this.stderrQueue.push(bytes);
        this.outputQueue.push(bytes);
    }
    finish(status) {
        this.exitStatus = status;
        if (this.exitResolve) {
            this.exitResolve(status);
            this.exitResolve = null;
        }
    }
}
class HostRuntimeProcess {
    constructor() {
        this.stdoutQueue = new StreamQueue();
        this.stderrQueue = new StreamQueue();
        this.outputQueue = new StreamQueue();
        this.stdinQueue = new StdinQueue();
        this.exitResolve = null;
        this.exitStatus = { code: 0 };
        this.closed = false;
        this.procId = Math.floor(Math.random() * 100000) + 200000;
        this.killResolve = null;
        this.killSettled = false;
        this.exitPromise = new Promise((resolve) => {
            this.exitResolve = resolve;
        });
        this.killPromise = new Promise((resolve) => {
            this.killResolve = resolve;
        });
    }
    pid() {
        return this.procId;
    }
    wait() {
        return this.exitStatus;
    }
    exit() {
        return this.exitPromise;
    }
    writeStdin(data) {
        const bytes = coerceBytes(data, new TextEncoder());
        this.stdinQueue.push(bytes);
        return bytes.length;
    }
    readStdout(maxBytes) {
        return this.stdoutQueue.read(maxBytes);
    }
    readStderr(maxBytes) {
        return this.stderrQueue.read(maxBytes);
    }
    readOutput(maxBytes) {
        return this.outputQueue.read(maxBytes);
    }
    stdinStream() {
        return this.stdinQueue.stream;
    }
    stdoutStream() {
        return this.stdoutQueue.stream;
    }
    stderrStream() {
        return this.stderrQueue.stream;
    }
    outputStream() {
        return this.outputQueue.stream;
    }
    kill(signal) {
        if (this.closed)
            return;
        this.finish({ code: 128, signal });
    }
    close() {
        this.closed = true;
    }
    writeStdout(bytes) {
        this.stdoutQueue.push(bytes);
        this.outputQueue.push(bytes);
    }
    writeStderr(bytes) {
        this.stderrQueue.push(bytes);
        this.outputQueue.push(bytes);
    }
    finish(status) {
        this.exitStatus = status;
        if (this.exitResolve) {
            this.exitResolve(status);
            this.exitResolve = null;
        }
        if (!this.killSettled && this.killResolve) {
            this.killResolve(status);
            this.killResolve = null;
            this.killSettled = true;
        }
    }
    waitForKill() {
        return this.killPromise;
    }
}
class StreamQueue {
    constructor() {
        this.queue = [];
        this.controller = null;
        this.stream = new ReadableStream({
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
    push(bytes) {
        if (this.controller) {
            this.controller.enqueue(bytes);
            return;
        }
        this.queue.push(bytes);
    }
    read(maxBytes) {
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
    constructor() {
        this.queue = [];
        this.stream = new WritableStream({
            write: (chunk) => {
                const bytes = coerceBytes(chunk, new TextEncoder());
                this.queue.push(bytes);
            },
        });
    }
    push(bytes) {
        this.queue.push(bytes);
    }
}
class ExitSignal extends Error {
    constructor(code) {
        super("Process exit");
        this.code = code;
    }
}
const DEFAULT_PATH = "/bin:/usr/bin:/usr/local/bin";
const MAX_SHEBANG_DEPTH = 8;
function resolveSpawnTarget(options) {
    const cwd = normalizePath(options.cwd || "/");
    const initialProgram = options.program;
    const initialArgs = options.args;
    const env = options.env ?? {};
    let runtimeLookup = lookupRuntime(options.commandRuntimes, initialProgram);
    let executablePath = resolveExecutablePath(options.fs, cwd, env, initialProgram);
    // POSIX-compatible fallback: allow mapped runtime even when no executable exists in PATH.
    if (!executablePath) {
        if (runtimeLookup) {
            return {
                kind: "runtime",
                program: runtimeLookup.key,
                args: initialArgs,
                runtime: runtimeLookup.runtime,
            };
        }
        if (isNodeProgram(initialProgram)) {
            return {
                kind: "native",
                program: initialProgram,
                args: initialArgs,
            };
        }
        return { kind: "not-found" };
    }
    let currentProgram = executablePath;
    let currentArgs = initialArgs;
    let depth = 0;
    while (depth < MAX_SHEBANG_DEPTH) {
        depth += 1;
        const shebang = parseShebang(options.fs, currentProgram);
        if (!shebang) {
            runtimeLookup =
                lookupRuntime(options.commandRuntimes, currentProgram) ??
                    lookupRuntime(options.commandRuntimes, basename(currentProgram));
            if (runtimeLookup) {
                return {
                    kind: "runtime",
                    program: runtimeLookup.key,
                    args: [currentProgram, ...currentArgs],
                    runtime: runtimeLookup.runtime,
                };
            }
            return {
                kind: "native",
                program: currentProgram,
                args: currentArgs,
            };
        }
        const interpreter = resolveShebangInterpreter(options.fs, options.commandRuntimes, cwd, env, shebang);
        if (!interpreter) {
            return { kind: "not-found" };
        }
        currentArgs = [...interpreter.args, currentProgram, ...currentArgs];
        runtimeLookup =
            lookupRuntime(options.commandRuntimes, interpreter.program) ??
                lookupRuntime(options.commandRuntimes, basename(interpreter.program));
        if (runtimeLookup) {
            return {
                kind: "runtime",
                program: runtimeLookup.key,
                args: currentArgs,
                runtime: runtimeLookup.runtime,
            };
        }
        executablePath = resolveExecutablePath(options.fs, cwd, env, interpreter.program);
        if (!executablePath) {
            return { kind: "not-found" };
        }
        currentProgram = executablePath;
    }
    return { kind: "not-found" };
}
function lookupRuntime(commandRuntimes, program) {
    const direct = commandRuntimes[program];
    if (direct) {
        return { key: program, runtime: direct };
    }
    const base = basename(program);
    if (!base || base === program) {
        return null;
    }
    const byBase = commandRuntimes[base];
    if (byBase) {
        return { key: base, runtime: byBase };
    }
    return null;
}
function resolveExecutablePath(fs, cwd, env, program) {
    if (program.includes("/")) {
        const candidate = resolvePath(cwd, program);
        return fileType(fs, candidate) === "file" ? candidate : null;
    }
    const pathValue = env.PATH || DEFAULT_PATH;
    const pathEntries = pathValue
        .split(":")
        .map((entry) => entry.trim())
        .filter((entry) => entry.length > 0);
    for (const entry of pathEntries) {
        const base = entry.startsWith("/") ? normalizePath(entry) : resolvePath(cwd, entry);
        const candidate = normalizePath(`${base}/${program}`);
        if (fileType(fs, candidate) === "file") {
            return candidate;
        }
    }
    return null;
}
function parseShebang(fs, path) {
    let bytes;
    try {
        bytes = fs.readFile(path);
    }
    catch {
        return null;
    }
    const head = bytes.slice(0, Math.min(bytes.length, 1024));
    const line = new TextDecoder().decode(head).split(/\r?\n/, 1)[0] ?? "";
    if (!line.startsWith("#!")) {
        return null;
    }
    const command = line.slice(2).trim();
    if (!command) {
        return null;
    }
    return command.split(/\s+/).filter((part) => part.length > 0);
}
function resolveShebangInterpreter(fs, commandRuntimes, cwd, env, tokens) {
    const parts = [...tokens];
    let interpreter = parts.shift();
    if (!interpreter) {
        return null;
    }
    if (interpreter === "/usr/bin/env" || interpreter === "env") {
        interpreter = parts.shift();
        if (!interpreter) {
            return null;
        }
    }
    const runtime = lookupRuntime(commandRuntimes, interpreter);
    if (runtime) {
        return { program: interpreter, args: parts };
    }
    const resolvedProgram = resolveExecutablePath(fs, cwd, env, interpreter);
    if (!resolvedProgram) {
        return null;
    }
    return { program: resolvedProgram, args: parts };
}
function createCommandNotFoundProcess(program) {
    const handle = new HostRuntimeProcess();
    queueMicrotask(() => {
        handle.writeStderr(new TextEncoder().encode(`${program}: command not found\n`));
        handle.finish({ code: 127 });
    });
    return new Process(handle);
}
function isNodeProgram(program) {
    return program === "node" || program.endsWith("/node");
}
function createRequire(options) {
    return function require(specifier) {
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
        const resolved = resolveRequireSpecifier(options.fs, options.cwdRef.value, options.projectRoot, specifier);
        const filename = resolveModuleFile(options.fs, resolved);
        const cached = options.moduleCache.get(filename);
        if (cached) {
            return cached.exports;
        }
        const record = { exports: {} };
        options.moduleCache.set(filename, record);
        const code = options.decoder.decode(options.fs.readFile(filename));
        if (filename.endsWith(".json")) {
            record.exports = JSON.parse(code);
            return record.exports;
        }
        const module = { exports: record.exports };
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
function runModuleCode(options) {
    const wrapper = new Function("exports", "require", "module", "__filename", "__dirname", "process", "console", "Buffer", options.code);
    wrapper(options.module.exports, options.require, options.module, options.filename, options.dirname, options.process, options.console, options.buffer);
}
function createFsModule(fs, cwdRef, decoder, encoder) {
    return {
        readFileSync: (path, options) => {
            const resolved = resolvePath(cwdRef.value, path);
            const data = fs.readFile(resolved);
            const encoding = typeof options === "string" ? options : options?.encoding;
            if (encoding === "utf8" || encoding === "utf-8") {
                return decoder.decode(data);
            }
            return data;
        },
        writeFileSync: (path, data, options) => {
            const resolved = resolvePath(cwdRef.value, path);
            const bytes = typeof data === "string" ? encoder.encode(data) : data;
            fs.writeFile(resolved, bytes, options);
        },
        readdirSync: (path) => {
            const resolved = resolvePath(cwdRef.value, path);
            return fs.readdir(resolved);
        },
        mkdirSync: (path, options) => {
            const resolved = resolvePath(cwdRef.value, path);
            fs.mkdir(resolved, options);
        },
        rmSync: (path, options) => {
            const resolved = resolvePath(cwdRef.value, path);
            fs.rm(resolved, options);
        },
        renameSync: (from, to) => {
            fs.rename(resolvePath(cwdRef.value, from), resolvePath(cwdRef.value, to));
        },
        statSync: (path) => {
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
function createPathModule(cwdRef) {
    return {
        join: (...parts) => normalizePath(parts.join("/")),
        resolve: (...parts) => {
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
        isAbsolute: (path) => path.startsWith("/"),
    };
}
function resolvePath(cwd, path) {
    if (path.startsWith("/")) {
        return normalizePath(path);
    }
    return normalizePath(`${cwd}/${path}`);
}
function resolveRequireSpecifier(fs, cwd, projectRoot, specifier) {
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
function resolveBareModuleSpecifier(options) {
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
function hasModuleEntry(fs, root) {
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
function resolveModuleFile(fs, path) {
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
function fileType(fs, path) {
    try {
        const stat = fs.stat(path);
        if (stat.fileType === "file") {
            return "file";
        }
        if (stat.fileType === "dir") {
            return "dir";
        }
        return null;
    }
    catch {
        return null;
    }
}
function normalizePath(path) {
    const absolute = path.startsWith("/");
    const parts = path.split("/").filter((part) => part.length > 0);
    const stack = [];
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
function dirname(path) {
    if (path === "/") {
        return "/";
    }
    const parts = path.split("/").filter((part) => part.length > 0);
    parts.pop();
    return parts.length === 0 ? "/" : `/${parts.join("/")}`;
}
function basename(path) {
    const parts = path.split("/").filter((part) => part.length > 0);
    return parts[parts.length - 1] ?? "";
}
function extname(path) {
    const base = basename(path);
    const index = base.lastIndexOf(".");
    if (index <= 0) {
        return "";
    }
    return base.slice(index);
}
function coerceBytes(input, encoder) {
    if (input instanceof Uint8Array) {
        return input;
    }
    if (typeof input === "string") {
        return encoder.encode(input);
    }
    return encoder.encode(String(input));
}
function makeConsole(proc, encoder) {
    const print = (writer) => {
        return (...args) => {
            const message = args.map(String).join(" ");
            writer(encoder.encode(`${message}\n`));
        };
    };
    return {
        log: print((bytes) => proc.writeStdout(bytes)),
        info: print((bytes) => proc.writeStdout(bytes)),
        warn: print((bytes) => proc.writeStderr(bytes)),
        error: print((bytes) => proc.writeStderr(bytes)),
    };
}
function readDekaJson(fs, projectRoot, decoder) {
    const file = normalizePath(`${projectRoot}/deka.json`);
    try {
        const raw = decoder.decode(fs.readFile(file));
        const parsed = JSON.parse(raw);
        return parsed && typeof parsed === "object" ? parsed : null;
    }
    catch {
        return null;
    }
}
function parseServeArgs(args) {
    let entryArg;
    let portArg;
    for (let i = 0; i < args.length; i += 1) {
        const arg = args[i];
        if (arg === "--port" && i + 1 < args.length) {
            const parsed = Number(args[i + 1]);
            if (Number.isFinite(parsed) && parsed > 0) {
                portArg = parsed;
            }
            i += 1;
            continue;
        }
        if (!arg.startsWith("-") && !entryArg) {
            entryArg = arg;
        }
    }
    return { entryArg, portArg };
}
function normalizeDekaArgs(args) {
    if (args.length === 0) {
        return [];
    }
    const first = args[0];
    if (first === "deka" || first.endsWith("/deka")) {
        return args.slice(1);
    }
    return args;
}
async function runPhpxEntry(options) {
    let source;
    try {
        const bytes = options.context.fs.readFile(options.entry);
        source = options.decoder.decode(bytes);
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        return {
            code: 1,
            stderr: `deka run: unable to read ${options.entry}: ${message}\n`,
        };
    }
    const mode = options.mode ?? (options.entry.endsWith(".phpx") ? "phpx" : "php");
    const result = await options.phpRuntime.run(source, mode, {
        filename: options.entry,
        cwd: dirname(options.entry),
    });
    const diagnostics = (result.diagnostics ?? [])
        .map((diag) => `[${diag.severity}] ${diag.message}`)
        .join("\n");
    return {
        code: result.ok ? 0 : 1,
        stdout: result.stdout ?? "",
        stderr: [result.stderr ?? "", diagnostics]
            .filter((part) => part.length > 0)
            .join(result.stderr && diagnostics ? "\n" : ""),
    };
}
class BufferShim {
    static from(value) {
        if (typeof value === "string") {
            return new TextEncoder().encode(value);
        }
        if (value instanceof Uint8Array) {
            return value;
        }
        return new Uint8Array(value);
    }
}
