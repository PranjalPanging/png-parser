/* tslint:disable */
/* eslint-disable */

export function capacity_js(contents: Uint8Array, mode_str: string): number;

export function delete_js(contents: Uint8Array, password?: string | null): Uint8Array;

export function fingerprint_js(contents: Uint8Array): string;

export function hide_js(contents: Uint8Array, file_bytes: Uint8Array, filename: string, password: string | null | undefined, mode_str: string, expires_days?: bigint | null, expires_hours?: bigint | null, expires_minutes?: bigint | null, expires_seconds?: bigint | null): Uint8Array;

export function info_js(contents: Uint8Array, password?: string | null): string;

export function reencrypt_js(contents: Uint8Array, old_password: string, new_password: string): Uint8Array;

export function reveal_js(contents: Uint8Array, password?: string | null): Uint8Array;

export function verify_js(contents: Uint8Array, password: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly capacity_js: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly delete_js: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly fingerprint_js: (a: number, b: number, c: number) => void;
    readonly hide_js: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: bigint, n: number, o: bigint, p: number, q: bigint, r: number, s: bigint) => void;
    readonly info_js: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly reencrypt_js: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly reveal_js: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly verify_js: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly __wbindgen_export: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
