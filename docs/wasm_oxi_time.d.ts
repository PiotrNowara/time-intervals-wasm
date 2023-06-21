/* tslint:disable */
/* eslint-disable */
/**
* @param {HTMLInputElement} file_input
* @param {string} start_date_prop_name
* @param {string} end_date_prop_name
*/
export function analyze_file(file_input: HTMLInputElement, start_date_prop_name: string, end_date_prop_name: string): void;
/**
* @param {string} input_string
* @param {string} start_date_prop_name
* @param {string} end_date_prop_name
*/
export function analyze_string(input_string: string, start_date_prop_name: string, end_date_prop_name: string): void;
/**
* @param {string} input_string
*/
export function analyze_csv_string(input_string: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly analyze_file: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly analyze_string: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly analyze_csv_string: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly _dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hdebc3c848bc1cd0e: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
