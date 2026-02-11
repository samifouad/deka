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

export class PhpRuntimeAdapter {
  private readonly bridge: PhpHostBridge;
  private readonly executor: PhpRuntimeExecutor | null;

  constructor(options: PhpRuntimeAdapterOptions) {
    this.bridge = options.bridge;
    this.executor = options.executor ?? null;
  }

  async run(source: string, mode: PhpRunMode = "phpx", context: PhpRunContext = {}): Promise<PhpRunResult> {
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
        },
      };
    }

    if (this.executor) {
      return this.executor.run(source, mode, context);
    }

    return this.notImplemented(source, mode, context);
  }

  private notImplemented(source: string, mode: PhpRunMode, context: PhpRunContext): PhpRunResult {
    return {
      ok: false,
      stdout: "",
      stderr: "",
      diagnostics: [
        {
          severity: "error",
          file: context.filename ?? "unknown.phpx",
          message:
            "PHPX browser execution adapter is not wired yet. Use Node shim demo for now.",
        },
      ],
      meta: {
        mode,
        filename: context.filename ?? "unknown.phpx",
        sourceBytes: new TextEncoder().encode(source).length,
        phase: "not_implemented",
      },
    };
  }
}

export function createPhpRuntimeAdapter(options: PhpRuntimeAdapterOptions): PhpRuntimeAdapter {
  return new PhpRuntimeAdapter(options);
}

export function createPhpRuntimeAdapterFromBridgeOptions(
  options: Parameters<typeof createPhpHostBridge>[0]
): PhpRuntimeAdapter {
  const bridge = createPhpHostBridge(options);
  return new PhpRuntimeAdapter({ bridge });
}
