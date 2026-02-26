import { PhpHostBridge, createPhpHostBridge } from "./phpx_host_bridge.js";
export type PhpRunMode = "phpx" | "phpx_internal" | "php";
export type PhpRunContext = {
    filename?: string;
    cwd?: string;
    env?: Record<string, string>;
};
export type PhpRunDiagnostic = {
    severity: "error" | "warning" | "info";
    message: string;
    file?: string;
    line?: number;
    column?: number;
};
export type PhpRunResult = {
    ok: boolean;
    stdout: string;
    stderr: string;
    diagnostics: PhpRunDiagnostic[];
    meta: Record<string, unknown>;
};
export type PhpRuntimeAdapterOptions = {
    bridge: PhpHostBridge;
    executor?: PhpRuntimeExecutor;
};
export type PhpRuntimeExecutor = {
    run(source: string, mode: PhpRunMode, context: PhpRunContext): Promise<PhpRunResult>;
};
export declare class PhpRuntimeAdapter {
    private readonly bridge;
    private readonly executor;
    constructor(options: PhpRuntimeAdapterOptions);
    run(source: string, mode?: PhpRunMode, context?: PhpRunContext): Promise<PhpRunResult>;
    private notImplemented;
}
export declare function createPhpRuntimeAdapter(options: PhpRuntimeAdapterOptions): PhpRuntimeAdapter;
export declare function createPhpRuntimeAdapterFromBridgeOptions(options: Parameters<typeof createPhpHostBridge>[0]): PhpRuntimeAdapter;
