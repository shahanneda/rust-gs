#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&format!( $( $t )* ).into());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!($($t)*);
            }
        }
    }
}

#[macro_export]
macro_rules! float_32_array {
    ($arr:expr) => {{
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let arr_location = $arr.as_ptr() as u32 / 4;
        let array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + $arr.len() as u32);
        array
    }};
}
#[macro_export]
macro_rules! uint_16_array {
    ($arr:expr) => {{
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let arr_location = $arr.as_ptr() as u32 / 2;
        let array = js_sys::Uint16Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + $arr.len() as u32);
        array
    }};
}

pub fn sigmoid(val: f32) -> f32 {
    1.0 / (1.0 + (-val).exp())
}
