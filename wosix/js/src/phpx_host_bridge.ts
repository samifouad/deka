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

export type BridgeCallOutput =
  | { ok: true; value: unknown }
  | { ok: false; error: string };

export type HostFs = {
  readFile(path: string): Uint8Array;
  writeFile(path: string, data: Uint8Array, options?: Record<string, unknown>): void;
  readdir(path: string): string[];
  mkdir(path: string, options?: Record<string, unknown>): void;
  rm(path: string, options?: Record<string, unknown>): void;
  rename(from: string, to: string): void;
  stat(path: string): { size: number; fileType: string };
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

const SERVER_CAPS: HostCapabilities = {
  fs: true,
  net: true,
  processEnv: true,
  clockRandom: true,
  db: true,
  wasmImports: true,
};

const WOSIX_CAPS: HostCapabilities = {
  fs: true,
  net: true,
  processEnv: false,
  clockRandom: true,
  db: false,
  wasmImports: true,
};

export class PhpHostBridge {
  private readonly fs: HostFs;
  private readonly projectRoot: string;
  private cwdValue: string;
  private readonly env: Record<string, string>;
  private readonly target: HostTarget;
  private readonly caps: HostCapabilities;
  private readonly stdio;

  constructor(options: PhpHostBridgeOptions) {
    this.fs = options.fs;
    this.projectRoot = normalizePath(options.projectRoot ?? "/");
    this.cwdValue = normalizePath(options.cwd ?? this.projectRoot);
    this.env = { ...(options.env ?? {}) };
    this.target = options.target ?? "wosix";
    const base = this.target === "server" ? SERVER_CAPS : WOSIX_CAPS;
    this.caps = { ...base, ...(options.capabilities ?? {}) };
    this.stdio = options.stdio;
  }

  call(input: BridgeCallInput): BridgeCallOutput {
    const kind = input.kind.trim();
    const action = input.action.trim();
    const payload = input.payload ?? {};

    const needed = capabilityForBridgeKind(kind);
    if (needed && !this.allow(needed)) {
      return {
        ok: false,
        error: capabilityError(this.target, needed, kind, action),
      };
    }

    try {
      if (kind === "fs") {
        return { ok: true, value: this.callFs(action, payload) };
      }
    if (kind === "process" || kind === "env") {
      return { ok: true, value: this.callProcessEnv(action, payload) };
    }
      if (kind === "clock" || kind === "random") {
        return { ok: true, value: this.callClockRandom(kind, action, payload) };
      }
      return {
        ok: false,
        error: `bridge error: unknown bridge kind '${kind}'`,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      return { ok: false, error: `bridge error: ${message}` };
    }
  }

  private callFs(action: string, payload: Record<string, unknown>): unknown {
    const path = this.requirePath(payload.path);
    const resolved = this.resolvePath(path);
    if (action === "readFile") {
      return { data: this.fs.readFile(resolved) };
    }
    if (action === "writeFile") {
      const data = payload.data;
      const bytes = toBytes(data);
      this.fs.writeFile(resolved, bytes, asRecord(payload.options));
      return { ok: true };
    }
    if (action === "readdir") {
      return { entries: this.fs.readdir(resolved) };
    }
    if (action === "mkdir") {
      this.fs.mkdir(resolved, asRecord(payload.options));
      return { ok: true };
    }
    if (action === "rm") {
      this.fs.rm(resolved, asRecord(payload.options));
      return { ok: true };
    }
    if (action === "rename") {
      const toPath = this.requirePath(payload.to);
      this.fs.rename(resolved, this.resolvePath(toPath));
      return { ok: true };
    }
    if (action === "stat") {
      return this.fs.stat(resolved);
    }
    throw new Error(`unknown fs action '${action}'`);
  }

  private callProcessEnv(action: string, payload: Record<string, unknown>): unknown {
    if (action === "cwd") {
      return { cwd: this.cwdValue };
    }
    if (action === "chdir") {
      const path = this.requirePath(payload.path);
      this.cwdValue = this.resolvePath(path);
      return { cwd: this.cwdValue };
    }
    if (action === "envGet") {
      const key = requireString(payload.key, "key");
      return { value: this.env[key] ?? null };
    }
    if (action === "envSet") {
      const key = requireString(payload.key, "key");
      const value = requireString(payload.value, "value");
      this.env[key] = value;
      return { ok: true };
    }
    if (action === "envAll") {
      return { env: { ...this.env } };
    }
    if (action === "writeStdout") {
      const chunk = toBytes(payload.data);
      this.stdio?.writeStdout?.(chunk);
      return { bytes: chunk.length };
    }
    if (action === "writeStderr") {
      const chunk = toBytes(payload.data);
      this.stdio?.writeStderr?.(chunk);
      return { bytes: chunk.length };
    }
    if (action === "readStdin") {
      const maxBytes = payload.maxBytes ? Number(payload.maxBytes) : undefined;
      const data = this.stdio?.readStdin?.(maxBytes);
      return { data: data ?? new Uint8Array(0) };
    }
    throw new Error(`unknown process/env action '${action}'`);
  }

  private callClockRandom(
    kind: string,
    action: string,
    payload: Record<string, unknown>
  ): unknown {
    if (kind === "clock" && (action === "now" || action === "nowMs")) {
      return { nowMs: Date.now() };
    }
    if (kind === "random" && (action === "bytes" || action === "randomBytes")) {
      const len = Number(payload.length ?? 16);
      if (!Number.isFinite(len) || len < 0 || len > 65536) {
        throw new Error("random length must be between 0 and 65536");
      }
      const out = new Uint8Array(len);
      crypto.getRandomValues(out);
      return { data: out };
    }
    throw new Error(`unknown ${kind} action '${action}'`);
  }

  private allow(cap: keyof HostCapabilities): boolean {
    return this.caps[cap];
  }

  private requirePath(value: unknown): string {
    return requireString(value, "path");
  }

  private resolvePath(path: string): string {
    if (path.startsWith("/")) {
      return normalizePath(path);
    }
    return normalizePath(`${this.cwdValue}/${path}`);
  }
}

export function createPhpHostBridge(options: PhpHostBridgeOptions): PhpHostBridge {
  return new PhpHostBridge(options);
}

function capabilityForBridgeKind(kind: string): keyof HostCapabilities | null {
  if (kind === "fs") return "fs";
  if (kind === "net" || kind === "tcp" || kind === "tls") return "net";
  if (kind === "process" || kind === "env") return "processEnv";
  if (kind === "clock" || kind === "random") return "clockRandom";
  if (kind === "db" || kind === "postgres" || kind === "mysql" || kind === "sqlite") return "db";
  if (kind === "wasm" || kind === "component") return "wasmImports";
  return null;
}

function capabilityError(
  host: HostTarget,
  capability: keyof HostCapabilities,
  kind: string,
  action: string
): string {
  return `CapabilityError(host=${host}, capability=${capability}, kind=${kind}, action=${action}): operation is not available in this host profile`;
}

function requireString(value: unknown, name: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${name} must be a non-empty string`);
  }
  return value;
}

function asRecord(value: unknown): Record<string, unknown> | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return undefined;
  }
  return value as Record<string, unknown>;
}

function toBytes(input: unknown): Uint8Array {
  if (input instanceof Uint8Array) {
    return input;
  }
  if (typeof input === "string") {
    return new TextEncoder().encode(input);
  }
  if (Array.isArray(input)) {
    const out = new Uint8Array(input.length);
    for (let i = 0; i < input.length; i++) {
      const n = Number(input[i]);
      out[i] = Number.isFinite(n) ? n & 0xff : 0;
    }
    return out;
  }
  throw new Error("data must be Uint8Array|string|number[]");
}

function normalizePath(path: string): string {
  const absolute = path.startsWith("/");
  const parts = path.split("/").filter((part) => part.length > 0);
  const stack: string[] = [];
  for (const part of parts) {
    if (part === ".") continue;
    if (part === "..") {
      stack.pop();
      continue;
    }
    stack.push(part);
  }
  return `${absolute ? "/" : ""}${stack.join("/")}` || "/";
}
