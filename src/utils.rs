use js_sys::{Float32Array, Uint32Array, Uint8Array, WebAssembly};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};
extern crate nalgebra_glm as glm;

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut(f32)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn float32_array_from_vec(vec: &[f32]) -> Float32Array {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let location: u32 = vec.as_ptr() as u32 / 4;
    return Float32Array::new(&memory_buffer).subarray(location, location + vec.len() as u32);
}

pub fn uint32_array_from_vec(vec: &[u32]) -> Uint32Array {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let location: u32 = vec.as_ptr() as u32 / 4;
    return Uint32Array::new(&memory_buffer).subarray(location, location + vec.len() as u32);
}

pub fn invert_row(mat: &mut glm::Mat4, row: usize) {
    mat[row + 0] = -mat[row + 0];
    mat[row + 4] = -mat[row + 4];
    mat[row + 8] = -mat[row + 8];
    mat[row + 12] = -mat[row + 12];
}

/// Log current Wasm memory usage with a label (debug builds only)
#[cfg(target_arch = "wasm32")]
pub fn debug_memory(label: &str) {
    // SAFETY: wasm_bindgen::memory() always returns the current module's memory object in Wasm.
    let mem = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap();
    let buffer = mem.buffer();
    let bytes = Uint8Array::new(&buffer).length() as f64;
    crate::log!("mem {}: {:.2} MB", label, bytes / (1024.0 * 1024.0));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn debug_memory(_label: &str) {}
