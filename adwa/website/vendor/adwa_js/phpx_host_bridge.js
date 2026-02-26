import { handleDbBridge } from "./phpx_db_bridge.js";
const SERVER_CAPS = {
    fs: true,
    net: true,
    processEnv: true,
    clockRandom: true,
    db: true,
    wasmImports: true,
};
const ADWA_CAPS = {
    fs: true,
    net: true,
    processEnv: false,
    clockRandom: true,
    db: false,
    wasmImports: true,
};
export class PhpHostBridge {
    constructor(options) {
        this.fs = options.fs;
        this.projectRoot = normalizePath(options.projectRoot ?? "/");
        this.cwdValue = normalizePath(options.cwd ?? this.projectRoot);
        this.env = { ...(options.env ?? {}) };
        this.target = options.target ?? "adwa";
        const base = this.target === "server" ? SERVER_CAPS : ADWA_CAPS;
        this.caps = { ...base, ...(options.capabilities ?? {}) };
        this.netAllowlist = (options.net?.allowlist ?? []).map((entry) => String(entry || "").trim()).filter(Boolean);
        this.stdio = options.stdio;
    }
    call(input) {
        const kind = input.kind.trim();
        const action = input.action.trim();
        const payload = input.payload ?? {};
        const needed = capabilityForBridgeCall(kind, action);
        if (needed && !this.allow(needed)) {
            const suggestion = capabilitySuggestion(this.target, needed, kind, action);
            return {
                ok: false,
                error: capabilityError(this.target, needed, kind, action, suggestion),
                code: "CAPABILITY_DENIED",
                info: {
                    kind: "capability_denied",
                    host: this.target,
                    capability: needed,
                    bridgeKind: kind,
                    action,
                    suggestion,
                },
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
            if (kind === "net" || kind === "tcp" || kind === "tls") {
                return { ok: true, value: this.callNet(kind, action, payload) };
            }
            if (kind === "db" || kind === "sqlite" || kind === "postgres" || kind === "mysql") {
                return { ok: true, value: handleDbBridge(kind, action, payload) };
            }
            return {
                ok: false,
                error: `bridge error: unknown bridge kind '${kind}'`,
                code: "UNKNOWN_KIND",
                info: {
                    kind: "unknown_kind",
                    host: this.target,
                    bridgeKind: kind,
                    action,
                    suggestion: "Use one of: fs, process/env, clock/random, net/tcp/tls, db, wasm.",
                },
            };
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            return {
                ok: false,
                error: `bridge error: ${message}`,
                code: "BRIDGE_RUNTIME_ERROR",
                info: {
                    kind: "runtime_error",
                    host: this.target,
                    bridgeKind: kind,
                    action,
                },
            };
        }
    }
    describeHost() {
        return {
            target: this.target,
            capabilities: { ...this.caps },
            netAllowlist: [...this.netAllowlist],
        };
    }
    callFs(action, payload) {
        const op = this.normalizeFsAction(action);
        const path = this.requirePath(payload.path);
        const resolved = this.resolvePath(path);
        if (op === "readFile") {
            return { data: this.fs.readFile(resolved) };
        }
        if (op === "writeFile") {
            const data = payload.data;
            const bytes = toBytes(data);
            this.fs.writeFile(resolved, bytes, asRecord(payload.options));
            return { ok: true };
        }
        if (op === "readdir") {
            return { entries: this.fs.readdir(resolved) };
        }
        if (op === "mkdir") {
            this.fs.mkdir(resolved, asRecord(payload.options));
            return { ok: true };
        }
        if (op === "rm") {
            this.fs.rm(resolved, asRecord(payload.options));
            return { ok: true };
        }
        if (op === "rename") {
            const toPath = this.requirePath(payload.to);
            this.fs.rename(resolved, this.resolvePath(toPath));
            return { ok: true };
        }
        if (op === "stat") {
            return this.fs.stat(resolved);
        }
        throw new Error(`unknown fs action '${action}'`);
    }
    normalizeFsAction(action) {
        if (action === "read_file")
            return "readFile";
        if (action === "write_file")
            return "writeFile";
        if (action === "read_dir")
            return "readdir";
        return action;
    }
    callProcessEnv(action, payload) {
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
        if (action === "envUnset") {
            const key = requireString(payload.key, "key");
            delete this.env[key];
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
    callClockRandom(kind, action, payload) {
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
    callNet(kind, action, payload) {
        if (action === "fetch") {
            const url = requireString(payload.url, "url");
            if (!this.isAllowedNetworkUrl(url)) {
                throw new Error(`NetworkDenied: '${url}' is not in net.allowlist`);
            }
            throw new Error("net.fetch is not implemented in this host yet");
        }
        throw new Error(`unknown ${kind} action '${action}'`);
    }
    allow(cap) {
        return this.caps[cap];
    }
    requirePath(value) {
        return requireString(value, "path");
    }
    resolvePath(path) {
        if (path.startsWith("/")) {
            return normalizePath(path);
        }
        return normalizePath(`${this.cwdValue}/${path}`);
    }
    isAllowedNetworkUrl(rawUrl) {
        if (this.netAllowlist.length === 0) {
            return false;
        }
        let url;
        try {
            url = new URL(rawUrl);
        }
        catch {
            return false;
        }
        const host = `${url.protocol}//${url.host}`;
        for (const rule of this.netAllowlist) {
            if (rule.endsWith("*")) {
                const prefix = rule.slice(0, -1);
                if (host.startsWith(prefix) || rawUrl.startsWith(prefix)) {
                    return true;
                }
                continue;
            }
            if (rawUrl === rule || host === rule) {
                return true;
            }
        }
        return false;
    }
}
export function createPhpHostBridge(options) {
    return new PhpHostBridge(options);
}
function capabilityForBridgeKind(kind) {
    if (kind === "fs")
        return "fs";
    if (kind === "net" || kind === "tcp" || kind === "tls")
        return "net";
    if (kind === "process" || kind === "env")
        return "processEnv";
    if (kind === "clock" || kind === "random")
        return "clockRandom";
    if (kind === "db" || kind === "postgres" || kind === "mysql" || kind === "sqlite")
        return "db";
    if (kind === "wasm" || kind === "component")
        return "wasmImports";
    return null;
}
function capabilityForBridgeCall(kind, action) {
    // Keep stdio usable in restricted hosts (like adwa) without opening full
    // process/env mutation APIs.
    if ((kind === "process" || kind === "env") &&
        (action === "writeStdout" || action === "writeStderr" || action === "readStdin")) {
        return null;
    }
    return capabilityForBridgeKind(kind);
}
function capabilityError(host, capability, kind, action, suggestion) {
    const base = `CapabilityError(host=${host}, capability=${capability}, kind=${kind}, action=${action}): operation is not available in this host profile`;
    if (!suggestion || suggestion.length === 0) {
        return base;
    }
    return `${base}\nhelp: ${suggestion}`;
}
function capabilitySuggestion(host, capability, kind, action) {
    if (capability === "db") {
        return "Database capability is disabled for this host profile. Enable db capability or provide a compatible db adapter.";
    }
    if (capability === "processEnv") {
        return "Process/env APIs are restricted in adwa. Pass config through runtime context/env injection instead.";
    }
    if (capability === "net") {
        return "Network access is restricted by host policy. Add an allowlist entry or proxy through a server endpoint.";
    }
    if (capability === "wasmImports") {
        return "Direct wasm imports are restricted in this host profile. Use runtime adapter-managed modules.";
    }
    if (capability === "fs") {
        return "Filesystem capability is disabled. Mount a writable virtual FS before calling fs APIs.";
    }
    if (capability === "clockRandom") {
        return "Clock/random capability is disabled. Inject deterministic values from host context.";
    }
    return `Operation ${kind}.${action} is unavailable for host '${host}'.`;
}
function requireString(value, name) {
    if (typeof value !== "string" || value.length === 0) {
        throw new Error(`${name} must be a non-empty string`);
    }
    return value;
}
function asRecord(value) {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        return undefined;
    }
    return value;
}
function toBytes(input) {
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
function normalizePath(path) {
    const absolute = path.startsWith("/");
    const parts = path.split("/").filter((part) => part.length > 0);
    const stack = [];
    for (const part of parts) {
        if (part === ".")
            continue;
        if (part === "..") {
            stack.pop();
            continue;
        }
        stack.push(part);
    }
    return `${absolute ? "/" : ""}${stack.join("/")}` || "/";
}
