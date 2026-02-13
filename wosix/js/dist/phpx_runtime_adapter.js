import { createPhpHostBridge } from "./phpx_host_bridge.js";
export class PhpRuntimeAdapter {
    constructor(options) {
        this.bridge = options.bridge;
        this.executor = options.executor ?? null;
    }
    async run(source, mode = "phpx", context = {}) {
        const hostTarget = this.bridge.hostTarget();
        const hostCapabilities = this.bridge.capabilities();
        const fsProbe = this.bridge.call({
            kind: "fs",
            action: "stat",
            payload: { path: context.cwd ?? "/" },
        });
        if (!fsProbe.ok) {
            const bridgeDiagnostic = toBridgeDiagnostic(fsProbe.error);
            return {
                ok: false,
                stdout: "",
                stderr: "",
                diagnostics: [
                    {
                        severity: "error",
                        message: bridgeDiagnostic.message,
                    },
                ],
                meta: {
                    mode,
                    filename: context.filename ?? "unknown.phpx",
                    phase: "preflight",
                    hostTarget,
                    hostCapabilities,
                    error: bridgeDiagnostic,
                },
            };
        }
        if (this.executor) {
            const result = await this.executor.run(source, mode, context);
            return {
                ...result,
                meta: {
                    ...result.meta,
                    hostTarget,
                    hostCapabilities,
                },
            };
        }
        return this.notImplemented(source, mode, context);
    }
    notImplemented(source, mode, context) {
        return {
            ok: false,
            stdout: "",
            stderr: "",
            diagnostics: [
                {
                    severity: "error",
                    file: context.filename ?? "unknown.phpx",
                    message: "PHPX browser execution adapter is not wired yet. Use Node shim demo for now.",
                },
            ],
            meta: {
                mode,
                filename: context.filename ?? "unknown.phpx",
                sourceBytes: new TextEncoder().encode(source).length,
                phase: "not_implemented",
                hostTarget: this.bridge.hostTarget(),
                hostCapabilities: this.bridge.capabilities(),
            },
        };
    }
}
export function createPhpRuntimeAdapter(options) {
    return new PhpRuntimeAdapter(options);
}
export function createPhpRuntimeAdapterFromBridgeOptions(options) {
    const bridge = createPhpHostBridge(options);
    return new PhpRuntimeAdapter({ bridge });
}
function toBridgeDiagnostic(error) {
    const cap = parseCapabilityError(error);
    if (cap) {
        return {
            kind: "capability_error",
            message: `Capability '${cap.capability}' is unavailable in '${cap.host}' host profile for ` +
                `${cap.bridgeKind}.${cap.action}.`,
            help: capabilityHelp(cap.capability),
            host: cap.host,
            capability: cap.capability,
            bridgeKind: cap.bridgeKind,
            action: cap.action,
        };
    }
    return {
        kind: "bridge_error",
        message: error,
    };
}
function parseCapabilityError(error) {
    const match = /^CapabilityError\(host=([^,]+), capability=([^,]+), kind=([^,]+), action=([^)]+)\):/.exec(error);
    if (!match) {
        return null;
    }
    return {
        host: match[1].trim(),
        capability: match[2].trim(),
        bridgeKind: match[3].trim(),
        action: match[4].trim(),
    };
}
function capabilityHelp(capability) {
    if (capability === "fs") {
        return "Enable fs in the host profile or avoid filesystem APIs in this execution mode.";
    }
    if (capability === "processEnv") {
        return "Use explicit config/input values instead of process/env in this host profile.";
    }
    if (capability === "db") {
        return "Use server host profile for database calls, or mock/stub DB in browser.";
    }
    if (capability === "net") {
        return "Use an allowlisted fetch bridge or move network access to server host profile.";
    }
    return "Switch to a host profile that provides this capability or remove this API call.";
}
