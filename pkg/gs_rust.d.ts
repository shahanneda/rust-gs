/* tslint:disable */
/* eslint-disable */
export function editor_get_objects(): string;
export function editor_select(kind: string, idx: number): void;
export function editor_deselect(): void;
export function editor_set_hidden(kind: string, idx: number, hidden: boolean): void;
export function editor_rename(kind: string, idx: number, name: string): void;
export function editor_delete(kind: string, idx: number): boolean;
export function editor_duplicate(kind: string, idx: number): boolean;
/**
 * Live (shader-only) tint preview for a splat object.
 */
export function editor_set_tint_preview(idx: number, r: number, g: number, b: number, strength: number): void;
/**
 * Bake the current preview tint into the splat colors (undoable).
 */
export function editor_apply_tint(idx: number): void;
/**
 * Set the flat color of a mesh object.
 */
export function editor_set_mesh_color(idx: number, r: number, g: number, b: number): void;
export function editor_set_eraser_config(radius: number, preview: boolean): void;
export function editor_set_slice_config(separation: number, target_selected: boolean, mode: number): void;
export function editor_undo(): boolean;
export function editor_add_splat_from_url(url: string, name: string): Promise<void>;
export function editor_add_mesh_from_url(url: string, name: string, scale: number): Promise<void>;
export function editor_add_primitive(kind: string, r: number, g: number, b: number, size: number): void;
/**
 * Render one clean frame and return its RGBA pixels (bottom-up, WebGL
 * convention — the JS side flips). Also remembers the view-projection
 * matrix so a mask computed on this frame can be projected back onto the
 * splats later, even if the camera has since moved.
 */
export function editor_capture_frame(): Uint8Array;
/**
 * Given a segmentation mask over the captured frame (row-major, top-down,
 * one byte per pixel, non-zero = selected), find the matching splats.
 * mode 0: extract them into a new object (returns its index).
 * mode 1: erase them (undoable; returns number of erased splats).
 * Returns -1 when nothing matched.
 */
export function editor_apply_mask(mask: Uint8Array, mask_w: number, mask_h: number, mode: number): number;
/**
 * Number of visible splats the eraser would delete at its current position.
 */
export function editor_pending_erase_count(): number;
export function start(): Promise<void>;
export function initThreadPool(num_threads: number): Promise<any>;
export function wbg_rayon_start_worker(receiver: number): void;
export class IntoUnderlyingByteSource {
  private constructor();
  free(): void;
  start(controller: ReadableByteStreamController): void;
  pull(controller: ReadableByteStreamController): Promise<any>;
  cancel(): void;
  readonly type: string;
  readonly autoAllocateChunkSize: number;
}
export class IntoUnderlyingSink {
  private constructor();
  free(): void;
  write(chunk: any): Promise<any>;
  close(): Promise<any>;
  abort(reason: any): Promise<any>;
}
export class IntoUnderlyingSource {
  private constructor();
  free(): void;
  pull(controller: ReadableStreamDefaultController): Promise<any>;
  cancel(): void;
}
export class wbg_rayon_PoolBuilder {
  private constructor();
  free(): void;
  mainJS(): string;
  numThreads(): number;
  receiver(): number;
  build(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly editor_get_objects: () => [number, number];
  readonly editor_select: (a: number, b: number, c: number) => void;
  readonly editor_deselect: () => void;
  readonly editor_set_hidden: (a: number, b: number, c: number, d: number) => void;
  readonly editor_rename: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly editor_delete: (a: number, b: number, c: number) => number;
  readonly editor_duplicate: (a: number, b: number, c: number) => number;
  readonly editor_set_tint_preview: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly editor_apply_tint: (a: number) => void;
  readonly editor_set_mesh_color: (a: number, b: number, c: number, d: number) => void;
  readonly editor_set_eraser_config: (a: number, b: number) => void;
  readonly editor_set_slice_config: (a: number, b: number, c: number) => void;
  readonly editor_undo: () => number;
  readonly editor_add_splat_from_url: (a: number, b: number, c: number, d: number) => any;
  readonly editor_add_mesh_from_url: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly editor_add_primitive: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly editor_capture_frame: () => [number, number];
  readonly editor_apply_mask: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly editor_pending_erase_count: () => number;
  readonly start: () => void;
  readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
  readonly intounderlyingbytesource_type: (a: number) => [number, number];
  readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
  readonly intounderlyingbytesource_start: (a: number, b: any) => void;
  readonly intounderlyingbytesource_pull: (a: number, b: any) => any;
  readonly intounderlyingbytesource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
  readonly intounderlyingsource_pull: (a: number, b: any) => any;
  readonly intounderlyingsource_cancel: (a: number) => void;
  readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
  readonly intounderlyingsink_write: (a: number, b: any) => any;
  readonly intounderlyingsink_close: (a: number) => any;
  readonly intounderlyingsink_abort: (a: number, b: any) => any;
  readonly __wbg_wbg_rayon_poolbuilder_free: (a: number, b: number) => void;
  readonly wbg_rayon_poolbuilder_mainJS: (a: number) => any;
  readonly wbg_rayon_poolbuilder_numThreads: (a: number) => number;
  readonly wbg_rayon_poolbuilder_receiver: (a: number) => number;
  readonly wbg_rayon_poolbuilder_build: (a: number) => void;
  readonly initThreadPool: (a: number) => any;
  readonly wbg_rayon_start_worker: (a: number) => void;
  readonly memory: WebAssembly.Memory;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_7: WebAssembly.Table;
  readonly closure2_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure319_externref_shim: (a: number, b: number, c: any) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h186c289636e08325: (a: number, b: number) => void;
  readonly closure326_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure360_externref_shim: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
  readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
* @param {WebAssembly.Memory} memory - Deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
* @param {WebAssembly.Memory} memory - Deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
