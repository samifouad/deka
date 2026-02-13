export type HostTarget = "server" | "wosix";
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
export type BridgeCallOutput = {
    ok: true;
    value: unknown;
} | {
    ok: false;
    error: string;
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
    private readonly stdio;
    constructor(options: PhpHostBridgeOptions);
    call(input: BridgeCallInput): BridgeCallOutput;
    capabilities(): HostCapabilities;
    hostTarget(): HostTarget;
    private callFs;
    private callProcessEnv;
    private callClockRandom;
    private allow;
    private requirePath;
    private resolvePath;
}
export declare function createPhpHostBridge(options: PhpHostBridgeOptions): PhpHostBridge;
