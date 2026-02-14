export type HostTarget = "server" | "adwa";
export type HostCapabilities = {
    fs: boolean;
    net: boolean;
    processEnv: boolean;
    clockRandom: boolean;
    db: boolean;
    wasmImports: boolean;
};
export type BridgeCallInput = {
    kind: string;
    action: string;
    payload?: Record<string, unknown>;
};
export type BridgeErrorInfo = {
    kind: "capability_denied" | "unknown_kind" | "invalid_input" | "runtime_error";
    host?: HostTarget;
    capability?: keyof HostCapabilities;
    bridgeKind?: string;
    action?: string;
    suggestion?: string;
};
export type BridgeCallOutput = {
    ok: true;
    value: unknown;
} | {
    ok: false;
    error: string;
    code?: string;
    info?: BridgeErrorInfo;
};
export type HostFs = {
    readFile(path: string): Uint8Array;
    writeFile(path: string, data: Uint8Array, options?: Record<string, unknown>): void;
    readdir(path: string): string[];
    mkdir(path: string, options?: Record<string, unknown>): void;
    rm(path: string, options?: Record<string, unknown>): void;
    rename(from: string, to: string): void;
    stat(path: string): {
        size: number;
        fileType: string;
    };
};
export type PhpHostBridgeOptions = {
    fs: HostFs;
    target?: HostTarget;
    projectRoot?: string;
    cwd?: string;
    env?: Record<string, string>;
    capabilities?: Partial<HostCapabilities>;
    net?: {
        allowlist?: string[];
    };
    stdio?: {
        writeStdout?: (chunk: Uint8Array) => void;
        writeStderr?: (chunk: Uint8Array) => void;
        readStdin?: (maxBytes?: number) => Uint8Array | null;
    };
};
export declare class PhpHostBridge {
    private readonly fs;
    private readonly projectRoot;
    private cwdValue;
    private readonly env;
    private readonly target;
    private readonly caps;
    private readonly netAllowlist;
    private readonly stdio;
    constructor(options: PhpHostBridgeOptions);
    call(input: BridgeCallInput): BridgeCallOutput;
    describeHost(): {
        target: HostTarget;
        capabilities: HostCapabilities;
        netAllowlist: string[];
    };
    private callFs;
    private normalizeFsAction;
    private callProcessEnv;
    private callClockRandom;
    private callNet;
    private allow;
    private requirePath;
    private resolvePath;
    private isAllowedNetworkUrl;
}
export declare function createPhpHostBridge(options: PhpHostBridgeOptions): PhpHostBridge;
