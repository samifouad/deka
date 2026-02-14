/* tslint:disable */
/* eslint-disable */

/**
 * Format a validation error with beautiful Rust/Gleam-style output
 *
 * This function creates consistent error messages across all Deka systems:
 * - deka-runtime (native Rust)
 * - deka-edge (native Rust)
 * - playground (WASM in browser)
 * - CLI tools (WASM in Bun/Node)
 *
 * # Arguments
 *
 * * `code` - The source code containing the error
 * * `file_path` - Path to the file (e.g., "handler.ts")
 * * `error_kind` - Category of error (e.g., "Invalid Import", "Type Error")
 * * `line_num` - Line number (1-indexed)
 * * `col_num` - Column number (1-indexed)
 * * `message` - Error message
 * * `help` - Help text explaining how to fix
 * * `underline_length` - Number of characters to underline (for ^^^)
 *
 * # Example
 *
 * ```rust
 * use deka_validation::format_validation_error;
 *
 * let error = format_validation_error(
 *     "import { serve } from 'deka/invalid';",
 *     "handler.ts",
 *     "Invalid Import",
 *     1,
 *     26,
 *     "Module 'deka/invalid' not found",
 *     "Available modules: deka, deka/router, deka/sqlite",
 *     12
 * );
 *
 * // Produces:
 * // Validation Error
 * // ❌ Invalid Import
 * //
 * // ┌─ handler.ts:1:26
 * // │
 * //   1 │ import { serve } from 'deka/invalid';
 * //     │                          ^^^^^^^^^^^^ Module 'deka/invalid' not found
 * // │
 * // = help: Available modules: deka, deka/router, deka/sqlite
 * // │
 * // └─
 * ```
 */
export function format_validation_error(code: string, file_path: string, error_kind: string, line_num: number, col_num: number, message: string, help: string, underline_length: number): string;

export function format_validation_error_extended(code: string, file_path: string, error_kind: string, line_num: number, col_num: number, message: string, help: string, underline_length: number, severity: string, docs_link?: string | null): string;

export function format_validation_error_with_suggestion(code: string, file_path: string, error_kind: string, line_num: number, col_num: number, message: string, help: string, underline_length: number, severity: string, docs_link?: string | null, suggestion?: string | null): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly php_alloc: (a: number) => number;
    readonly php_free: (a: number, b: number) => void;
    readonly php_run: (a: number, b: number) => number;
    readonly format_validation_error: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => [number, number];
    readonly format_validation_error_extended: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number, q: number) => [number, number];
    readonly format_validation_error_with_suggestion: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number, q: number, r: number, s: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
