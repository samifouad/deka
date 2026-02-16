import type { PhpRunContext, PhpRunMode, PhpRunResult, PhpRuntimeExecutor } from "./phpx_runtime_adapter.js";
import type { PhpHostBridge } from "./phpx_host_bridge.js";
export type PhpRuntimeWasmExecutorOptions = {
    moduleUrl: string;
    wasmUrl?: string;
    bridge: PhpHostBridge;
};
export declare class PhpRuntimeWasmExecutor implements PhpRuntimeExecutor {
    private readonly moduleUrl;
    private readonly wasmUrl?;
    private readonly bridge;
    private runtime;
    private readonly encoder;
    private readonly decoder;
    private runtimeMeta;
    constructor(options: PhpRuntimeWasmExecutorOptions);
    run(source: string, mode: PhpRunMode, context: PhpRunContext): Promise<PhpRunResult>;
    private ensureRuntime;
}
export declare function createPhpRuntimeWasmExecutor(options: PhpRuntimeWasmExecutorOptions): PhpRuntimeWasmExecutor;
