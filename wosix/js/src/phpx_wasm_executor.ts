import type {
  PhpRunContext,
  PhpRunMode,
  PhpRunResult,
  PhpRuntimeExecutor,
} from "./phpx_runtime_adapter.js";

type PhpRuntimeWasmExports = {
  memory: WebAssembly.Memory;
  php_alloc(size: number): number;
  php_free(ptr: number, size: number): void;
  php_run(ptr: number, len: number): number;
};

type PhpRuntimeWasmInit = (
  moduleOrPath?: string | URL | Request | Response | Promise<unknown> | { module_or_path: unknown }
) => Promise<PhpRuntimeWasmExports>;

export type PhpRuntimeWasmExecutorOptions = {
  moduleUrl: string;
};

export class PhpRuntimeWasmExecutor implements PhpRuntimeExecutor {
  private readonly moduleUrl: string;
  private runtime: PhpRuntimeWasmExports | null = null;
  private readonly encoder = new TextEncoder();
  private readonly decoder = new TextDecoder();

  constructor(options: PhpRuntimeWasmExecutorOptions) {
    this.moduleUrl = options.moduleUrl;
  }

  async run(source: string, mode: PhpRunMode, context: PhpRunContext): Promise<PhpRunResult> {
    try {
      const runtime = await this.ensureRuntime();
      const withMarker = withModeMarker(source, mode);
      const srcBytes = this.encoder.encode(withMarker);
      const srcPtr = runtime.php_alloc(srcBytes.length);
      let mem = new Uint8Array(runtime.memory.buffer);
      mem.set(srcBytes, srcPtr);

      const resultPtr = runtime.php_run(srcPtr, srcBytes.length);
      runtime.php_free(srcPtr, srcBytes.length);

      mem = new Uint8Array(runtime.memory.buffer);
      const outPtr = readU32(mem, resultPtr);
      const outLen = readU32(mem, resultPtr + 4);
      const outBytes = mem.subarray(outPtr, outPtr + outLen);
      const outJson = this.decoder.decode(outBytes);
      runtime.php_free(outPtr, outLen);

      let parsed: Record<string, unknown>;
      try {
        parsed = JSON.parse(outJson);
      } catch (err) {
        return formatFatalResult(
          context,
          `invalid php runtime payload: ${outJson}\n${err instanceof Error ? err.message : String(err)}`
        );
      }

      return {
        ok: Boolean(parsed.ok),
        stdout: String(parsed.stdout ?? ""),
        stderr: String(parsed.stderr ?? ""),
        diagnostics: [],
        meta: {
          mode,
          filename: context.filename ?? "unknown.phpx",
          status: parsed.status ?? 0,
          headers: Array.isArray(parsed.headers) ? parsed.headers : [],
          error: parsed.error ?? "",
        },
      };
    } catch (err) {
      return formatFatalResult(context, err instanceof Error ? err.message : String(err));
    }
  }

  private async ensureRuntime(): Promise<PhpRuntimeWasmExports> {
    if (this.runtime) {
      return this.runtime;
    }

    const mod = (await import(/* @vite-ignore */ this.moduleUrl)) as {
      default: PhpRuntimeWasmInit;
    };
    if (!mod || typeof mod.default !== "function") {
      throw new Error(`invalid php runtime module: ${this.moduleUrl}`);
    }
    const wasmUrl = this.moduleUrl.replace(/\.js$/, "_bg.wasm");
    this.runtime = await mod.default(wasmUrl);
    return this.runtime;
  }
}

export function createPhpRuntimeWasmExecutor(
  options: PhpRuntimeWasmExecutorOptions
): PhpRuntimeWasmExecutor {
  return new PhpRuntimeWasmExecutor(options);
}

function withModeMarker(source: string, mode: PhpRunMode): string {
  const trimmed = source.trimStart();
  if (trimmed.startsWith("/*__DEKA_PHPX__*/") || trimmed.startsWith("/*__DEKA_PHPX_INTERNAL__*/")) {
    return source;
  }
  if (mode === "phpx_internal") {
    return `/*__DEKA_PHPX_INTERNAL__*/\n${source}`;
  }
  if (mode === "phpx") {
    return `/*__DEKA_PHPX__*/\n${source}`;
  }
  return source;
}

function formatFatalResult(context: PhpRunContext, message: string): PhpRunResult {
  return {
    ok: false,
    stdout: "",
    stderr: "",
    diagnostics: [
      {
        severity: "error",
        file: context.filename ?? "unknown.phpx",
        message,
      },
    ],
    meta: {
      phase: "runtime_error",
      filename: context.filename ?? "unknown.phpx",
    },
  };
}

function readU32(bytes: Uint8Array, offset: number): number {
  return (
    bytes[offset] |
    (bytes[offset + 1] << 8) |
    (bytes[offset + 2] << 16) |
    (bytes[offset + 3] << 24)
  ) >>> 0;
}
