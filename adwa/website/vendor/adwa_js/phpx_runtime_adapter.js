import { createPhpHostBridge } from "./phpx_host_bridge.js";
export class PhpRuntimeAdapter {
    constructor(options) {
        this.bridge = options.bridge;
        this.executor = options.executor ?? null;
    }
    async run(source, mode = "phpx", context = {}) {
        const host = this.bridge.describeHost();
        const fsProbe = this.bridge.call({
            kind: "fs",
            action: "stat",
            payload: { path: context.cwd ?? "/" },
        });
        if (!fsProbe.ok) {
            return {
                ok: false,
                stdout: "",
                stderr: "",
                diagnostics: [
                    {
                        severity: "error",
                        message: fsProbe.error,
                    },
                ],
                meta: {
                    mode,
                    filename: context.filename ?? "unknown.phpx",
                    phase: "preflight",
                    host,
                    bridgeError: {
                        code: fsProbe.code ?? "",
                        info: fsProbe.info ?? null,
                    },
                },
            };
        }
        if (this.executor) {
            const out = await this.executor.run(source, mode, context);
            const nextMeta = {
                host,
                ...(out.meta ?? {}),
            };
            return {
                ...out,
                meta: nextMeta,
            };
        }
        return this.notImplemented(source, mode, context, host);
    }
    notImplemented(source, mode, context, host) {
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
                host,
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
